#!/usr/bin/env python3
"""Bounded, network-isolated rich attachment extraction worker."""

import base64
import json
import os
import re
import signal
import socket
import subprocess
import sys
import tempfile
import textwrap
import zipfile
from io import BytesIO
from select import select
from pathlib import Path
from xml.etree import ElementTree

from PIL import Image, ImageDraw, ImageFont


MAX_REQUEST_BYTES = 8 * 1024
MAX_SOURCE_BYTES = 25 * 1024 * 1024
MAX_OUTPUT_BYTES = 1 * 1024 * 1024
MAX_PREVIEW_BYTES = 2 * 1024 * 1024
MAX_CDR_BYTES = 2 * 1024 * 1024
MAX_CDR_PAGES = 4
MAX_CDR_PIXELS_PER_PAGE = 3 * 1024 * 1024
DOCX_PREVIEW_WIDTH = 1200
DOCX_PREVIEW_HEIGHT = 1500
DOCX_PREVIEW_MAX_LINES = 52
DOCX_CDR_WIDTH = 1024
DOCX_CDR_HEIGHT = 1320
DOCX_CDR_LINES_PER_PAGE = 44
MAX_ZIP_ENTRIES = 10_000
MAX_ZIP_UNCOMPRESSED_BYTES = 50 * 1024 * 1024
COMMAND_TIMEOUT_SECONDS = 20
MAIL_ROOT = Path("/mail").resolve()
SOCKET_PATH = Path(os.environ.get("MAKOSH_ATTACHMENT_EXTRACTOR_SOCKET", "/run/makosh-extractor/extractor.sock"))
TCP_BIND = os.environ.get("MAKOSH_ATTACHMENT_EXTRACTOR_TCP_BIND", "0.0.0.0:8788")
OCR_LANGUAGES_RAW = os.environ.get("MAKOSH_ATTACHMENT_OCR_LANGUAGES", "eng+rus")


class ExtractionError(Exception):
    """A sanitized error that is safe to return to the caller."""


def ocr_languages(value: str) -> str:
    languages = [language.strip().lower() for language in value.split("+") if language.strip()]
    if not 1 <= len(languages) <= 8 or any(
        not re.fullmatch(r"[a-z][a-z0-9_]{0,31}", language) for language in languages
    ):
        raise ValueError("invalid MAKOSH_ATTACHMENT_OCR_LANGUAGES")
    return "+".join(languages)


OCR_LANGUAGES = ocr_languages(OCR_LANGUAGES_RAW)
Image.MAX_IMAGE_PIXELS = MAX_CDR_PIXELS_PER_PAGE


def bounded_text(value: str) -> tuple[str, bool]:
    encoded = value.encode("utf-8")
    if len(encoded) <= MAX_OUTPUT_BYTES:
        return value, False
    return encoded[:MAX_OUTPUT_BYTES].decode("utf-8", errors="ignore"), True


def receive_request(connection: socket.socket) -> bytes:
    chunks = []
    size = 0
    while True:
        chunk = connection.recv(min(4 * 1024, MAX_REQUEST_BYTES + 1 - size))
        if not chunk:
            return b"".join(chunks)
        chunks.append(chunk)
        size += len(chunk)
        if size > MAX_REQUEST_BYTES or b"\n" in chunk:
            return b"".join(chunks).split(b"\n", 1)[0]


def source_path(value: object) -> Path:
    if not isinstance(value, str) or not value:
        raise ExtractionError("invalid_source_path")
    candidate = (MAIL_ROOT / value).resolve()
    if MAIL_ROOT not in candidate.parents or not candidate.is_file():
        raise ExtractionError("source_not_found")
    if candidate.stat().st_size > MAX_SOURCE_BYTES:
        raise ExtractionError("source_too_large")
    return candidate


def extract_docx(path: Path) -> str:
    try:
        with zipfile.ZipFile(path) as archive:
            entries = archive.infolist()
            if len(entries) > MAX_ZIP_ENTRIES:
                raise ExtractionError("document_too_complex")
            if sum(entry.file_size for entry in entries) > MAX_ZIP_UNCOMPRESSED_BYTES:
                raise ExtractionError("document_too_large")
            try:
                document = archive.read("word/document.xml")
            except KeyError as error:
                raise ExtractionError("invalid_docx") from error
    except zipfile.BadZipFile as error:
        raise ExtractionError("invalid_docx") from error

    try:
        root = ElementTree.fromstring(document)
    except ElementTree.ParseError as error:
        raise ExtractionError("invalid_docx") from error

    paragraphs = []
    for paragraph in root.findall(".//{*}p"):
        text = "".join(node.text or "" for node in paragraph.findall(".//{*}t"))
        if text:
            paragraphs.append(text)
    return "\n".join(paragraphs)


def extract_command(command: list[str]) -> str:
    try:
        result = subprocess.run(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=COMMAND_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        raise ExtractionError("extractor_timeout") from error
    if result.returncode != 0:
        raise ExtractionError("extractor_failed")
    return result.stdout.decode("utf-8", errors="replace")


def render_pdf_preview(path: Path) -> bytes:
    with tempfile.TemporaryDirectory(prefix="makosh-preview-", dir="/tmp") as temporary_dir:
        output = Path(temporary_dir) / "page"
        try:
            result = subprocess.run(
                [
                    "pdftoppm",
                    "-f", "1",
                    "-l", "1",
                    "-singlefile",
                    "-png",
                    "-scale-to-x", "1200",
                    "-scale-to-y", "-1",
                    str(path),
                    str(output),
                ],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
                timeout=COMMAND_TIMEOUT_SECONDS,
            )
        except subprocess.TimeoutExpired as error:
            raise ExtractionError("preview_timeout") from error
        if result.returncode != 0:
            raise ExtractionError("preview_failed")
        try:
            preview = (output.with_suffix(".png")).read_bytes()
        except OSError as error:
            raise ExtractionError("preview_failed") from error
    if not preview.startswith(b"\x89PNG\r\n\x1a\n") or len(preview) > MAX_PREVIEW_BYTES:
        raise ExtractionError("preview_invalid")
    return preview


def render_docx_preview(path: Path) -> bytes:
    text = extract_docx(path)
    lines = []
    truncated = False
    for paragraph in text.splitlines():
        clean = "".join(character for character in paragraph if character.isprintable() or character == " ")
        lines.extend(textwrap.wrap(clean, width=88, break_long_words=True, break_on_hyphens=False) or [""])
        if len(lines) >= DOCX_PREVIEW_MAX_LINES:
            truncated = True
            break
    if len(lines) > DOCX_PREVIEW_MAX_LINES:
        lines = lines[:DOCX_PREVIEW_MAX_LINES]
        truncated = True
    if truncated:
        lines[-1:] = ["[Preview truncated by Макошь safety limit]"]

    image = Image.new("RGB", (DOCX_PREVIEW_WIDTH, DOCX_PREVIEW_HEIGHT), "#f8fafc")
    draw = ImageDraw.Draw(image)
    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 24)
    title_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 32)
    draw.text((72, 64), "Макошь safe DOCX preview", fill="#0f172a", font=title_font)
    draw.line((72, 120, DOCX_PREVIEW_WIDTH - 72, 120), fill="#cbd5e1", width=2)
    draw.multiline_text((72, 156), "\n".join(lines), fill="#1e293b", font=font, spacing=10)
    output = BytesIO()
    image.save(output, format="PNG", optimize=True)
    preview = output.getvalue()
    if not preview.startswith(b"\x89PNG\r\n\x1a\n") or len(preview) > MAX_PREVIEW_BYTES:
        raise ExtractionError("preview_invalid")
    return preview


def disarm_docx(path: Path) -> bytes:
    """Rebuild a bounded DOCX text representation as a new image-only PDF."""
    text, _ = bounded_text(extract_docx(path))
    lines = []
    for paragraph in text.splitlines():
        clean = "".join(character for character in paragraph if character.isprintable() or character == " ")
        lines.extend(textwrap.wrap(clean, width=82, break_long_words=True, break_on_hyphens=False) or [""])
        if len(lines) >= MAX_CDR_PAGES * DOCX_CDR_LINES_PER_PAGE:
            break
    if not lines:
        lines = ["[No extractable text in DOCX]"]
    lines = lines[: MAX_CDR_PAGES * DOCX_CDR_LINES_PER_PAGE]

    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 20)
    title_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 28)
    images = []
    try:
        for page_index, start in enumerate(range(0, len(lines), DOCX_CDR_LINES_PER_PAGE), start=1):
            image = Image.new("RGB", (DOCX_CDR_WIDTH, DOCX_CDR_HEIGHT), "#ffffff")
            draw = ImageDraw.Draw(image)
            draw.text((56, 48), "Макошь content-disarmed DOCX", fill="#0f172a", font=title_font)
            draw.text((DOCX_CDR_WIDTH - 116, 54), f"{page_index}", fill="#64748b", font=font)
            draw.line((56, 102, DOCX_CDR_WIDTH - 56, 102), fill="#cbd5e1", width=2)
            draw.multiline_text(
                (56, 136),
                "\n".join(lines[start : start + DOCX_CDR_LINES_PER_PAGE]),
                fill="#1e293b",
                font=font,
                spacing=8,
            )
            images.append(image)
        output = BytesIO()
        images[0].save(
            output,
            format="PDF",
            save_all=True,
            append_images=images[1:],
            resolution=120.0,
            quality=75,
        )
        artifact = output.getvalue()
    finally:
        for image in images:
            image.close()

    if (
        not artifact.startswith(b"%PDF-")
        or b"%%EOF" not in artifact[-1024:]
        or len(artifact) > MAX_CDR_BYTES
    ):
        raise ExtractionError("cdr_invalid")
    return artifact


def disarm_pdf(path: Path) -> bytes:
    """Rasterize a bounded PDF and rebuild it without source PDF objects."""
    with tempfile.TemporaryDirectory(prefix="makosh-cdr-", dir="/tmp") as temporary_dir:
        output_prefix = Path(temporary_dir) / "page"
        try:
            result = subprocess.run(
                [
                    "pdftoppm",
                    "-f", "1",
                    "-l", str(MAX_CDR_PAGES),
                    "-png",
                    "-scale-to-x", "1024",
                    "-scale-to-y", "-1",
                    str(path),
                    str(output_prefix),
                ],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
                timeout=COMMAND_TIMEOUT_SECONDS,
            )
        except subprocess.TimeoutExpired as error:
            raise ExtractionError("cdr_timeout") from error
        if result.returncode != 0:
            raise ExtractionError("cdr_failed")

        rendered_pages = sorted(Path(temporary_dir).glob("page-*.png"))
        if not rendered_pages or len(rendered_pages) > MAX_CDR_PAGES:
            raise ExtractionError("cdr_page_limit")
        images = []
        try:
            for rendered_page in rendered_pages:
                with Image.open(rendered_page) as page:
                    if page.width * page.height > MAX_CDR_PIXELS_PER_PAGE:
                        raise ExtractionError("cdr_pixel_limit")
                    images.append(page.convert("RGB"))
            output = BytesIO()
            images[0].save(
                output,
                format="PDF",
                save_all=True,
                append_images=images[1:],
                resolution=120.0,
            )
            artifact = output.getvalue()
        finally:
            for image in images:
                image.close()

    if (
        not artifact.startswith(b"%PDF-")
        or b"%%EOF" not in artifact[-1024:]
        or len(artifact) > MAX_CDR_BYTES
    ):
        raise ExtractionError("cdr_invalid")
    return artifact


def extract(request: dict[str, object]) -> dict[str, object]:
    if request.get("operation") == "health":
        return {
            "status": "ok",
            "extractors": ["pdf", "pdf_preview", "pdf_cdr", "docx", "docx_preview", "docx_cdr", "ocr"],
            "ocr_languages": OCR_LANGUAGES,
        }
    operation = request.get("operation")
    if operation not in {"extract", "render_preview", "content_disarm"}:
        raise ExtractionError("unsupported_operation")

    path = source_path(request.get("source_path"))
    kind = request.get("kind")
    if operation == "content_disarm":
        if kind == "pdf":
            artifact = disarm_pdf(path)
        elif kind == "docx":
            artifact = disarm_docx(path)
        else:
            raise ExtractionError("unsupported_cdr_kind")
        return {
            "status": "completed",
            "content_type": "application/pdf",
            "artifact_base64": base64.b64encode(artifact).decode("ascii"),
        }
    if operation == "render_preview":
        if kind == "pdf":
            preview = render_pdf_preview(path)
        elif kind == "docx":
            preview = render_docx_preview(path)
        else:
            raise ExtractionError("unsupported_preview_kind")
        return {
            "status": "completed",
            "content_type": "image/png",
            "preview_base64": base64.b64encode(preview).decode("ascii"),
        }

    if kind == "pdf":
        text = extract_command(["pdftotext", "-q", "-enc", "UTF-8", str(path), "-"])
    elif kind == "docx":
        text = extract_docx(path)
    elif kind == "ocr":
        text = extract_command(["tesseract", str(path), "stdout", "-l", OCR_LANGUAGES])
    else:
        raise ExtractionError("unsupported_kind")

    text, truncated = bounded_text(text)
    return {
        "status": "completed",
        "text_base64": base64.b64encode(text.encode("utf-8")).decode("ascii"),
        "truncated": truncated,
    }


def handle(connection: socket.socket) -> None:
    connection.settimeout(COMMAND_TIMEOUT_SECONDS + 5)
    payload = receive_request(connection)
    if len(payload) > MAX_REQUEST_BYTES:
        response = {"status": "failed", "error": "request_too_large"}
    else:
        try:
            request = json.loads(payload.decode("utf-8"))
            if not isinstance(request, dict):
                raise ExtractionError("invalid_request")
            response = extract(request)
        except (json.JSONDecodeError, UnicodeDecodeError):
            response = {"status": "failed", "error": "invalid_request"}
        except ExtractionError as error:
            response = {"status": "failed", "error": str(error)}
        except Exception:
            response = {"status": "failed", "error": "internal_error"}
    connection.sendall(json.dumps(response, separators=(",", ":")).encode("utf-8"))


def main() -> int:
    SOCKET_PATH.parent.mkdir(parents=True, exist_ok=True)
    try:
        SOCKET_PATH.unlink()
    except FileNotFoundError:
        pass

    server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    server.bind(str(SOCKET_PATH))
    os.chmod(SOCKET_PATH, 0o660)
    server.listen(16)

    host, port = TCP_BIND.rsplit(":", 1)
    tcp_server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    tcp_server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    tcp_server.bind((host, int(port)))
    tcp_server.listen(16)

    def shutdown(_signal: int, _frame: object) -> None:
        server.close()
        tcp_server.close()

    signal.signal(signal.SIGTERM, shutdown)
    signal.signal(signal.SIGINT, shutdown)
    while True:
        try:
            ready, _, _ = select([server, tcp_server], [], [])
            listener = ready[0]
            connection, _address = listener.accept()
        except OSError:
            return 0
        with connection:
            handle(connection)


if __name__ == "__main__":
    sys.exit(main())

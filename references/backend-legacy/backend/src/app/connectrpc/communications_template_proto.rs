use chrono::{DateTime, Utc};
use makosh_connectrpc_contracts::makosh::communications::v1::{
    RenderedRichTemplate as ProtoRenderedRichTemplate, RichTemplate as ProtoRichTemplate,
    RichTemplateMailMergePreviewItem as ProtoRichTemplateMailMergePreviewItem,
    RichTemplateMailMergePreviewResponse,
};

use crate::domains::communications::templates::{
    CommunicationMergePreview, CommunicationMergePreviewItem, CommunicationTemplate,
    RenderedTemplate,
};

fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

pub(super) fn rich_template(item: CommunicationTemplate) -> ProtoRichTemplate {
    ProtoRichTemplate {
        template_id: item.template_id,
        name: item.name,
        subject_template: item.subject_template,
        body_template: item.body_template,
        variables: item.variables,
        placeholder_variables: item.placeholder_variables,
        undeclared_variables: item.undeclared_variables,
        unused_variables: item.unused_variables,
        malformed_placeholders: item.malformed_placeholders,
        language: item.language,
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}

pub(super) fn rendered_rich_template(item: RenderedTemplate) -> ProtoRenderedRichTemplate {
    ProtoRenderedRichTemplate {
        subject: item.subject,
        body: item.body,
        missing_variables: item.missing_variables,
        unresolved_variables: item.unresolved_variables,
        malformed_placeholders: item.malformed_placeholders,
        ..Default::default()
    }
}

pub(super) fn rich_template_mail_merge_preview(
    item: CommunicationMergePreview,
) -> RichTemplateMailMergePreviewResponse {
    RichTemplateMailMergePreviewResponse {
        template_id: item.template_id,
        row_count: item.row_count as u32,
        ready_count: item.ready_count as u32,
        blocked_count: item.blocked_count as u32,
        items: item
            .items
            .into_iter()
            .map(rich_template_mail_merge_preview_item)
            .collect(),
        ..Default::default()
    }
}

fn rich_template_mail_merge_preview_item(
    item: CommunicationMergePreviewItem,
) -> ProtoRichTemplateMailMergePreviewItem {
    ProtoRichTemplateMailMergePreviewItem {
        row_id: item.row_id,
        ready: item.ready,
        rendered: Some(rendered_rich_template(item.rendered)).into(),
        ..Default::default()
    }
}

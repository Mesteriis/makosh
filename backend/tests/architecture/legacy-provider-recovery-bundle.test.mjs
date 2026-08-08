import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('legacy provider recovery is an isolated first-party app build unit', async () => {
  const [
    inventory,
    adr,
    manifest,
    library,
    bundle,
    preparation,
    legacyConfiguration,
    legacyVault,
    privateFiles,
    receipt,
  ] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ).then(JSON.parse),
    readFile(
      new URL(
        'docs/adr/ADR-0321-legacy-provider-recovery-bundle-and-native-secret-custody.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/legacy-provider-recovery/Cargo.toml', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/legacy-provider-recovery/src/lib.rs', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/legacy-provider-recovery/src/bundle.rs', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/legacy-provider-recovery/src/preparation.rs', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/native/legacy-provider-recovery/src/legacy_configuration.rs',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/legacy-provider-recovery/src/legacy_vault.rs', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/legacy-provider-recovery/src/private_files.rs', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/legacy-provider-recovery/src/receipt.rs', PROJECT_ROOT),
      'utf8',
    ),
  ]);

  const bundleGate = inventory.slices.find(
    (slice) => slice.gate === 'legacy_provider_recovery_bundle_v1',
  );
  const custodyGate = inventory.slices.find(
    (slice) => slice.gate === 'legacy_provider_native_secret_custody_v1',
  );
  assert.deepEqual(bundleGate, {
    gate: 'legacy_provider_recovery_bundle_v1',
    role: 'app',
    owner: 'first_party_client',
    state: 'implemented',
    dependsOn: [],
  });
  assert.equal(custodyGate?.state, 'implemented');
  assert.match(adr, /Состояние реализации: implemented/);

  assert.match(manifest, /name = "makosh-legacy-provider-recovery"/);
  assert.match(manifest, /role = "app"/);
  assert.match(manifest, /owner = "first_party_client"/);
  assert.match(manifest, /surface = "maintenance_source_adapter"/);
  assert.match(manifest, /prepare = \["dep:postgres"\]/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|mail|telegram|whatsapp|zulip|kernel|gateway)/,
  );
  assert.doesNotMatch(manifest, /(?:reqwest|keyring|teloxide|tdlib|imap|oauth2)\s*=/);

  assert.match(library, /LegacyProviderRecoveryBundleV1/);
  assert.match(bundle, /GeneratedTelegramSessionStoreKey/);
  assert.match(bundle, /getrandom::getrandom/);
  assert.doesNotMatch(bundle, /oauth_token.*resolve_secret/s);
  assert.match(preparation, /\.read_only\(true\)/);
  assert.match(preparation, /IpAddr::V4\(Ipv4Addr::LOCALHOST\)/);
  assert.match(preparation, /decode_master_key_file/);
  assert.match(preparation, /validate_icloud_decryption/);
  assert.doesNotMatch(preparation, /Command::new|std::process::Command/);
  assert.match(
    legacyConfiguration,
    /provider_parser_rejects_shell_evaluation_and_unknown_keys/,
  );
  assert.doesNotMatch(legacyConfiguration, /Command::new|std::process::Command/);
  assert.match(legacyVault, /OpenFlags::SQLITE_OPEN_READ_ONLY/);
  assert.match(legacyVault, /Zeroizing/);
  assert.doesNotMatch(legacyVault, /keyring|Keychain|Security-framework/);
  assert.match(privateFiles, /BUNDLE_FILES: \[&str; 6\]/);
  assert.match(privateFiles, /metadata\.permissions\(\)\.mode\(\) & 0o077 != 0/);
  assert.match(privateFiles, /metadata\.file_type\(\)\.is_symlink\(\)/);
  assert.match(receipt, /RECEIPT_SCHEMA_REVISION: u16 = 1/);
  assert.match(receipt, /RecoveryStepStateV1::OutcomeUnknown/);
  assert.match(receipt, /explicit_retry/);
  assert.match(receipt, /create_new\(true\)/);
  assert.match(receipt, /file\.sync_all\(\)/);
  assert.match(receipt, /fs::rename\(&temporary, path\)/);
  assert.match(receipt, /options\.mode\(0o600\)/);
  assert.doesNotMatch(receipt, /email|username|secret_payload|oauth_code/);
});

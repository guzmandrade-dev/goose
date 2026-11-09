use anyhow::Result;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::acp::{
    configured_model_for_provider, extension_configs_to_mcp_servers, AcpProvider,
    AcpProviderConfig, ACP_CURRENT_MODEL,
};
use crate::config::search_path::SearchPaths;
use crate::config::Config;
use crate::providers::base::{
    current_working_dir, ProviderDef, ProviderDescriptor, ProviderMetadata,
};
use crate::providers::catalog::ProviderSetupMetadata;

pub(crate) const VIBE_ACP_PROVIDER_NAME: &str = "vibe-acp";
const VIBE_ACP_DOC_URL: &str = "https://docs.mistral.ai/vibe/code/use-vibe-in-other-ides";
pub(crate) const VIBE_ACP_BINARY: &str = "vibe-acp";

pub struct VibeAcpProvider;

impl goose_providers::base::ProviderDescriptor for VibeAcpProvider {
    fn metadata() -> ProviderMetadata {
        ProviderMetadata::new(
            VIBE_ACP_PROVIDER_NAME,
            "Mistral Vibe",
            "Use goose with Mistral Vibe via the vibe-acp adapter.",
            ACP_CURRENT_MODEL,
            vec![],
            VIBE_ACP_DOC_URL,
            vec![],
        )
        .with_setup_steps(vec![
            "Install the Mistral Vibe CLI, which provides the vibe-acp command",
            "Sign in with your Mistral account (run `vibe` to verify)",
        ])
        .with_setup(
            ProviderSetupMetadata::cli_agent(VIBE_ACP_BINARY, &["vibe-acp", "vibe"])
                .with_acp()
                .with_docs_url("https://github.com/mistralai/mistral-vibe")
                .show_only_when_installed(),
        )
    }
}

impl VibeAcpProvider {
    fn create(
        extensions: Vec<crate::config::ExtensionConfig>,
        working_dir: PathBuf,
        use_default_model: bool,
    ) -> BoxFuture<'static, Result<AcpProvider>> {
        Box::pin(async move {
            let config = Config::global();
            let resolved_command = SearchPaths::builder().with_npm().resolve(VIBE_ACP_BINARY)?;
            let goose_mode = config.resolve_goose_mode()?;
            let model = if use_default_model {
                ACP_CURRENT_MODEL.to_string()
            } else {
                configured_model_for_provider(config, VIBE_ACP_PROVIDER_NAME)
            };

            let session_config_options = if model == ACP_CURRENT_MODEL {
                vec![]
            } else {
                vec![("model".to_string(), model)]
            };

            let provider_config = AcpProviderConfig {
                command: resolved_command,
                args: vec![],
                env: vec![],
                env_remove: vec![],
                work_dir: working_dir,
                mcp_servers: extension_configs_to_mcp_servers(&extensions),
                session_mode_id: None,
                session_config_options,
                model_config_option_id: Some("model".to_string()),
                mode_mapping: HashMap::new(),
                notification_callback: None,
            };

            let metadata = Self::metadata();
            AcpProvider::connect(metadata.name, goose_mode, provider_config).await
        })
    }
}

impl ProviderDef for VibeAcpProvider {
    type Provider = AcpProvider;
    const SESSION_BOUND: bool = true;
    const RUNS_OWN_TOOL_LOOP: bool = true;

    fn from_env(
        extensions: Vec<crate::config::ExtensionConfig>,
        tls_config: Option<crate::providers::api_client::TlsConfig>,
    ) -> BoxFuture<'static, Result<AcpProvider>> {
        Self::from_env_with_working_dir(extensions, current_working_dir(), tls_config)
    }

    fn from_env_with_working_dir(
        extensions: Vec<crate::config::ExtensionConfig>,
        working_dir: PathBuf,
        _tls_config: Option<crate::providers::api_client::TlsConfig>,
    ) -> BoxFuture<'static, Result<AcpProvider>> {
        Self::create(extensions, working_dir, false)
    }

    fn from_env_with_default_model(
        extensions: Vec<crate::config::ExtensionConfig>,
        _tls_config: Option<crate::providers::api_client::TlsConfig>,
    ) -> BoxFuture<'static, Result<AcpProvider>> {
        Self::create(extensions, current_working_dir(), true)
    }
}

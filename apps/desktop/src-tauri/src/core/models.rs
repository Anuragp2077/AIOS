use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Mutex;

const MODEL_EVENT: &str = "aios:models";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Local,
    Cloud,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum ModelRole {
    General,
    Planner,
    Coder,
    Reviewer,
}

impl ModelRole {
    pub const ALL: [Self; 4] = [Self::General, Self::Planner, Self::Coder, Self::Reviewer];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Planner => "planner",
            Self::Coder => "coder",
            Self::Reviewer => "reviewer",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProviderInfo {
    pub id: String,
    pub name: String,
    pub kind: ProviderKind,
    pub endpoint: String,
    pub auth_env: Option<String>,
    pub enabled: bool,
    pub built_in: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub roles: Vec<ModelRole>,
    pub context_window: Option<u32>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSelection {
    pub role: ModelRole,
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsSnapshot {
    pub version: &'static str,
    pub provider_count: usize,
    pub model_count: usize,
    pub providers: Vec<ModelProviderInfo>,
    pub models: Vec<ModelInfo>,
    pub defaults: Vec<ModelSelection>,
}

struct ModelsInner {
    next_provider_id: u64,
    providers: Vec<ModelProviderInfo>,
    models: Vec<ModelInfo>,
    defaults: BTreeMap<ModelRole, ModelSelection>,
}

pub struct ModelsState {
    inner: Mutex<ModelsInner>,
}

impl ModelsState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(ModelsInner {
                next_provider_id: 3,
                providers: vec![
                    ModelProviderInfo {
                        id: "provider-local-ollama".to_string(),
                        name: "Ollama (Local)".to_string(),
                        kind: ProviderKind::Local,
                        endpoint: "http://127.0.0.1:11434/v1".to_string(),
                        auth_env: None,
                        enabled: true,
                        built_in: true,
                    },
                    ModelProviderInfo {
                        id: "provider-cloud-compatible".to_string(),
                        name: "OpenAI-Compatible (Cloud)".to_string(),
                        kind: ProviderKind::Cloud,
                        endpoint: "https://api.openai.com/v1".to_string(),
                        auth_env: Some("OPENAI_API_KEY".to_string()),
                        enabled: true,
                        built_in: true,
                    },
                ],
                models: Vec::new(),
                defaults: BTreeMap::new(),
            }),
        }
    }

    pub fn snapshot(&self) -> ModelsSnapshot {
        let inner = self.inner.lock().expect("model state lock poisoned");
        snapshot_from(&inner)
    }

    pub fn register_provider(
        &self,
        name: String,
        kind: ProviderKind,
        endpoint: String,
        auth_env: Option<String>,
    ) -> Result<ModelProviderInfo, String> {
        let name = name.trim().to_string();
        let endpoint = normalize_endpoint(&endpoint)?;
        let auth_env = normalize_auth_env(auth_env)?;

        if name.is_empty() {
            return Err("Provider name cannot be empty".to_string());
        }

        let mut inner = self.inner.lock().map_err(|_| "model state lock poisoned".to_string())?;
        if inner.providers.iter().any(|provider| provider.name.eq_ignore_ascii_case(&name)) {
            return Err("A provider with that name already exists".to_string());
        }

        let id = format!("provider-{:04}", inner.next_provider_id);
        inner.next_provider_id += 1;

        let provider = ModelProviderInfo {
            id,
            name,
            kind,
            endpoint,
            auth_env,
            enabled: true,
            built_in: false,
        };
        inner.providers.push(provider.clone());

        Ok(provider)
    }

    pub fn remove_provider(&self, id: &str) -> Result<ModelsSnapshot, String> {
        let mut inner = self.inner.lock().map_err(|_| "model state lock poisoned".to_string())?;
        let provider = inner
            .providers
            .iter()
            .find(|provider| provider.id == id)
            .cloned()
            .ok_or_else(|| format!("Provider '{id}' not found"))?;

        if provider.built_in {
            return Err("Built-in providers cannot be removed".to_string());
        }

        if inner.models.iter().any(|model| model.provider_id == id) {
            return Err("Remove the provider's models before removing the provider".to_string());
        }

        inner.providers.retain(|item| item.id != id);
        inner.defaults.retain(|_, selection| selection.provider_id != id);
        Ok(snapshot_from(&inner))
    }

    pub fn register_model(
        &self,
        provider_id: String,
        model_id: String,
        name: String,
        roles: Vec<ModelRole>,
        context_window: Option<u32>,
    ) -> Result<ModelInfo, String> {
        let provider_id = provider_id.trim().to_string();
        let model_id = model_id.trim().to_string();
        let name = name.trim().to_string();

        if provider_id.is_empty() || model_id.is_empty() || name.is_empty() {
            return Err("Provider, model ID and model name are required".to_string());
        }
        if model_id.len() > 120 || name.len() > 120 {
            return Err("Model ID and name must be 120 characters or fewer".to_string());
        }
        if context_window == Some(0) {
            return Err("Context window must be greater than 0".to_string());
        }

        let roles = if roles.is_empty() {
            vec![ModelRole::General]
        } else {
            roles
        };

        let mut inner = self.inner.lock().map_err(|_| "model state lock poisoned".to_string())?;
        if !inner.providers.iter().any(|provider| provider.id == provider_id && provider.enabled) {
            return Err(format!("Enabled provider '{provider_id}' not found"));
        }
        if inner.models.iter().any(|model| model.provider_id == provider_id && model.id == model_id) {
            return Err(format!("Model '{model_id}' is already registered for this provider"));
        }

        let model = ModelInfo {
            id: model_id,
            provider_id,
            name,
            roles,
            context_window,
            enabled: true,
        };
        inner.models.push(model.clone());
        Ok(model)
    }

    pub fn remove_model(&self, provider_id: &str, model_id: &str) -> Result<ModelsSnapshot, String> {
        let mut inner = self.inner.lock().map_err(|_| "model state lock poisoned".to_string())?;
        let before = inner.models.len();
        inner.models.retain(|model| !(model.provider_id == provider_id && model.id == model_id));
        if inner.models.len() == before {
            return Err(format!("Model '{provider_id}/{model_id}' not found"));
        }

        inner.defaults.retain(|_, selection| {
            !(selection.provider_id == provider_id && selection.model_id == model_id)
        });
        Ok(snapshot_from(&inner))
    }

    pub fn set_default(
        &self,
        role: ModelRole,
        provider_id: &str,
        model_id: &str,
    ) -> Result<ModelsSnapshot, String> {
        let mut inner = self.inner.lock().map_err(|_| "model state lock poisoned".to_string())?;
        let model = inner
            .models
            .iter()
            .find(|model| {
                model.provider_id == provider_id
                    && model.id == model_id
                    && model.enabled
                    && model.roles.contains(&role)
            })
            .ok_or_else(|| format!("Model '{provider_id}/{model_id}' cannot serve the '{}' role", role.as_str()))?;

        inner.defaults.insert(
            role,
            ModelSelection {
                role,
                provider_id: model.provider_id.clone(),
                model_id: model.id.clone(),
            },
        );
        Ok(snapshot_from(&inner))
    }

    pub fn clear_default(&self, role: ModelRole) -> Result<ModelsSnapshot, String> {
        let mut inner = self.inner.lock().map_err(|_| "model state lock poisoned".to_string())?;
        inner.defaults.remove(&role);
        Ok(snapshot_from(&inner))
    }

    pub fn model_exists(&self, provider_id: &str, model_id: &str) -> bool {
        let inner = self.inner.lock().expect("model state lock poisoned");
        inner.models.iter().any(|model| {
            model.provider_id == provider_id && model.id == model_id && model.enabled
        })
    }
}

#[tauri::command]
pub fn get_models(state: tauri::State<'_, ModelsState>) -> ModelsSnapshot {
    state.snapshot()
}

#[tauri::command]
pub fn register_model_provider(
    name: String,
    kind: ProviderKind,
    endpoint: String,
    auth_env: Option<String>,
    state: tauri::State<'_, ModelsState>,
    app: tauri::AppHandle,
) -> Result<ModelProviderInfo, String> {
    let provider = state.register_provider(name, kind, endpoint, auth_env)?;
    emit_snapshot(&app, &state.snapshot())?;
    Ok(provider)
}

#[tauri::command]
pub fn remove_model_provider(
    id: String,
    state: tauri::State<'_, ModelsState>,
    app: tauri::AppHandle,
) -> Result<ModelsSnapshot, String> {
    let snapshot = state.remove_provider(&id)?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn register_model(
    provider_id: String,
    model_id: String,
    name: String,
    roles: Vec<ModelRole>,
    context_window: Option<u32>,
    state: tauri::State<'_, ModelsState>,
    app: tauri::AppHandle,
) -> Result<ModelInfo, String> {
    let model = state.register_model(provider_id, model_id, name, roles, context_window)?;
    emit_snapshot(&app, &state.snapshot())?;
    Ok(model)
}

#[tauri::command]
pub fn remove_model(
    provider_id: String,
    model_id: String,
    state: tauri::State<'_, ModelsState>,
    app: tauri::AppHandle,
) -> Result<ModelsSnapshot, String> {
    let snapshot = state.remove_model(&provider_id, &model_id)?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn set_model_default(
    role: ModelRole,
    provider_id: String,
    model_id: String,
    state: tauri::State<'_, ModelsState>,
    app: tauri::AppHandle,
) -> Result<ModelsSnapshot, String> {
    let snapshot = state.set_default(role, &provider_id, &model_id)?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn clear_model_default(
    role: ModelRole,
    state: tauri::State<'_, ModelsState>,
    app: tauri::AppHandle,
) -> Result<ModelsSnapshot, String> {
    let snapshot = state.clear_default(role)?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

fn emit_snapshot(app: &tauri::AppHandle, snapshot: &ModelsSnapshot) -> Result<(), String> {
    tauri::Emitter::emit(app, MODEL_EVENT, snapshot)
        .map_err(|error| format!("failed to emit model event: {error}"))
}

fn snapshot_from(inner: &ModelsInner) -> ModelsSnapshot {
    ModelsSnapshot {
        version: "0.5.0",
        provider_count: inner.providers.len(),
        model_count: inner.models.len(),
        providers: inner.providers.clone(),
        models: inner.models.clone(),
        defaults: inner.defaults.values().cloned().collect(),
    }
}

fn normalize_endpoint(value: &str) -> Result<String, String> {
    let endpoint = value.trim().trim_end_matches('/').to_string();
    if endpoint.is_empty() {
        return Err("Provider endpoint cannot be empty".to_string());
    }
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        return Err("Provider endpoint must use http:// or https://".to_string());
    }
    Ok(endpoint)
}

fn normalize_auth_env(value: Option<String>) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim().to_string();
    if value.is_empty() {
        return Ok(None);
    }
    let valid = value
        .chars()
        .enumerate()
        .all(|(index, ch)| ch == '_' || ch.is_ascii_alphanumeric() && (index > 0 || !ch.is_ascii_digit()));
    if !valid {
        return Err("Auth environment variable must be a valid shell-style variable name".to_string());
    }
    Ok(Some(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_providers_are_available() {
        let state = ModelsState::new();
        let snapshot = state.snapshot();
        assert_eq!(snapshot.provider_count, 2);
        assert!(snapshot.providers.iter().any(|provider| provider.kind == ProviderKind::Local));
        assert!(snapshot.providers.iter().any(|provider| provider.kind == ProviderKind::Cloud));
    }

    #[test]
    fn models_require_existing_provider() {
        let state = ModelsState::new();
        let error = state
            .register_model(
                "missing".to_string(),
                "model-a".to_string(),
                "Model A".to_string(),
                vec![ModelRole::General],
                Some(8192),
            )
            .expect_err("model should require a provider");
        assert!(error.contains("not found"));
    }

    #[test]
    fn role_defaults_require_matching_role() {
        let state = ModelsState::new();
        state
            .register_model(
                "provider-local-ollama".to_string(),
                "qwen-example".to_string(),
                "Example Local Model".to_string(),
                vec![ModelRole::Coder],
                Some(32768),
            )
            .expect("model should register");

        let error = state
            .set_default(ModelRole::Planner, "provider-local-ollama", "qwen-example")
            .expect_err("coder-only model should not become planner default");
        assert!(error.contains("planner"));

        state
            .set_default(ModelRole::Coder, "provider-local-ollama", "qwen-example")
            .expect("coder default should work");
        assert_eq!(state.snapshot().defaults.len(), 1);
    }

    #[test]
    fn provider_removal_protects_models() {
        let state = ModelsState::new();
        let provider = state
            .register_provider(
                "Lab".to_string(),
                ProviderKind::Local,
                "http://127.0.0.1:9000/v1".to_string(),
                None,
            )
            .expect("provider should register");

        state
            .register_model(
                provider.id.clone(),
                "model-a".to_string(),
                "Model A".to_string(),
                vec![ModelRole::General],
                None,
            )
            .expect("model should register");

        let error = state.remove_provider(&provider.id).expect_err("provider with models cannot be removed");
        assert!(error.contains("models"));
    }

    #[test]
    fn invalid_auth_env_is_rejected() {
        let state = ModelsState::new();
        let error = state
            .register_provider(
                "Broken".to_string(),
                ProviderKind::Cloud,
                "https://example.com/v1".to_string(),
                Some("NOT VALID".to_string()),
            )
            .expect_err("invalid env var should fail");
        assert!(error.contains("environment"));
    }
}

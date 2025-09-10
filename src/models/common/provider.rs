use bon::Builder;
use serde::{Deserialize, Serialize};

/// OpenRouter routes requests to the best available providers for your model.
///
/// By default, [requests are load balanced](https://openrouter.ai/docs/features/provider-routing#price-based-load-balancing-default-strategy)
/// across the top providers to maximize uptime.
///
/// You can customize how your requests are routed by passing an instance of `ProviderPreferences` to the `chat` and `completion` methods.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct ProviderPreferences {
    /// List of provider slugs to try in order (e.g. `vec![Provider::Anthropic, Provider::OpenAI]`).
    ///
    /// [Learn more](https://openrouter.ai/docs/features/provider-routing#ordering-specific-providers)
    #[builder(with = FromIterator::from_iter)]
    order: Option<Vec<Provider>>,

    /// Whether to allow backup providers when the primary is unavailable.
    ///
    /// [Learn more](https://openrouter.ai/docs/features/provider-routing#disabling-fallbacks)
    #[builder(default = true)]
    allow_fallbacks: bool,

    /// Only use providers that support all parameters in your request.
    ///
    /// [Learn more](https://openrouter.ai/docs/features/provider-routing#requiring-providers-to-support-all-parameters-beta)
    #[builder(default = false)]
    require_parameters: bool,

    /// Control whether to use providers that may store data.
    ///
    /// [Learn more](https://openrouter.ai/docs/features/provider-routing#requiring-providers-to-comply-with-data-policies)
    #[builder(into, default = DataCollection::Allow)]
    data_collection: DataCollection,

    /// List of provider slugs to allow for this request.
    ///
    /// [Learn more](https://openrouter.ai/docs/features/provider-routing#allowing-only-specific-providers)
    #[builder(with = FromIterator::from_iter)]
    only: Option<Vec<Provider>>,

    /// List of provider slugs to skip for this request.
    ///
    /// [Learn more](https://openrouter.ai/docs/features/provider-routing#ignoring-providers)
    #[builder(with = FromIterator::from_iter)]
    ignore: Option<Vec<Provider>>,

    /// List of quantization levels to filter by (e.g. `[Quantization::Int4, Quantization::Int8]`).
    ///
    /// [Learn more](https://openrouter.ai/docs/features/provider-routing#quantization)
    #[builder(with = FromIterator::from_iter)]
    quantizations: Option<Vec<Quantization>>,

    /// Sort providers by price or throughput. (e.g. `Sort::Price` or `Sort::Throughput`).
    ///
    /// [Learn more](https://openrouter.ai/docs/features/provider-routing#provider-sorting)
    sort: Option<Sort>,

    /// The maximum pricing you want to pay for this request.
    ///
    /// [Learn more](https://openrouter.ai/docs/features/provider-routing#max-price)
    max_price: Option<MaxPrice>,
}

/// **By default**, OpenRouter load balances based on price, while taking uptime into account.
///
/// If you instead want to explicitly prioritize a particular provider attribute, you can include the sort field in `ProviderPreferences`. Load balancing will be disabled, and the router will try providers in order.
///
/// The three sort options are:
///     - `Sort::Price`: prioritize lowest price
///     - `Sort::Throughput`: prioritize highest throughput
///     - `Sort::Latency`: prioritize lowest latency
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    Price,
    Throughput,
    Latency,
}

/// List of providers available via OpenRouter.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    AnyScale,
    #[serde(rename = "Cent-ML")]
    CentMl,
    HuggingFace,
    #[serde(rename = "Hyperbolic 2")]
    Hyperbolic2,
    Lepton,
    #[serde(rename = "Lynn 2")]
    Lynn2,
    Lynn,
    Mancer,
    Modal,
    OctoAI,
    Recursal,
    Reflection,
    Replicate,
    #[serde(rename = "SambaNova 2")]
    SambaNova2,
    #[serde(rename = "SF Compute")]
    SfCompute,
    #[serde(rename = "Together 2")]
    Together2,
    #[serde(rename = "01.AI")]
    ZeroOneAI,
    AI21,
    AionLabs,
    Alibaba,
    #[serde(rename = "Amazon Bedrock")]
    AmazonBedrock,
    Anthropic,
    AtlasCloud,
    Atoma,
    Avian,
    Azure,
    BaseTen,
    Cerebras,
    Chutes,
    Cloudflare,
    Cohere,
    CrofAI,
    Crusoe,
    DeepInfra,
    DeepSeek,
    Enfer,
    Featherless,
    Fireworks,
    Friendli,
    GMICloud,
    Google,
    #[serde(rename = "Google AI Studio")]
    GoogleAIStudio,
    Groq,
    Hyperbolic,
    Inception,
    InferenceNet,
    Infermatic,
    Inflection,
    InoCloud,
    Kluster,
    Lambda,
    Liquid,
    #[serde(rename = "Mancer 2")]
    Mancer2,
    Meta,
    Minimax,
    Mistral,
    #[serde(rename = "Moonshot AI")]
    MoonshotAI,
    Morph,
    NCompass,
    Nebius,
    NextBit,
    Nineteen,
    Novita,
    OpenAI,
    OpenInference,
    Parasail,
    Perplexity,
    Phala,
    SambaNova,
    Stealth,
    Switchpoint,
    Targon,
    Together,
    Ubicloud,
    Venice,
    #[serde(rename = "xAI")]
    XAI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataCollection {
    Allow,
    Deny,
}

impl From<bool> for DataCollection {
    fn from(value: bool) -> Self {
        if value {
            DataCollection::Allow
        } else {
            DataCollection::Deny
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quantization {
    Int4,
    Int8,
    Fp4,
    Fp6,
    Fp8,
    Fp16,
    Bf16,
    Fp32,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxPrice {
    prompt: Option<i64>,
    completion: Option<i64>,
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn provider_preferences_serialization() {
        let target = json!({
            "order": ["Groq", "Azure", "Google AI Studio"],
            "allow_fallbacks": true,
            "require_parameters": false,
            "data_collection": "allow",
            "only": ["Groq"],
            "ignore": ["Together"],
            "quantizations": ["int4", "int8"],
            "sort": "price",
            "max_price": {
                "prompt": 1,
                "completion": 2
            }
        });

        let provider = ProviderPreferences::builder()
            .order([Provider::Groq, Provider::Azure, Provider::GoogleAIStudio])
            .only([Provider::Groq])
            .ignore([Provider::Together])
            .quantizations([Quantization::Int4, Quantization::Int8])
            .sort(Sort::Price)
            .max_price(MaxPrice {
                prompt: Some(1),
                completion: Some(2),
            })
            .build();

        let value = serde_json::to_value(provider).unwrap();

        assert_eq!(target, value);
    }
}

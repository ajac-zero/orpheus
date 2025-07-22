# Configuring Providers

By default, [requests are load-balanced](https://openrouter.ai/docs/features/provider-routing#load-balancing-default-strategy) across the top providers to maximize uptime.&#x20;

However, you can customize how your requests are routed with the `preferences` argument in the `chat` and `completions` builders. This allows you to modify your requests to best suit your use case. For example, you could:

* Set Groq as the only provider to get lightning-fast responses.
* Only allow providers that support tool-calling for Agents.&#x20;
* Only allow providers that do not collect your data for privacy concerns.
* Allow providers with a specific level of quantization to reduce costs.
* Give priority to providers with lower latency/cost, or higher throughput.


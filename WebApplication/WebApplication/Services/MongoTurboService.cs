using Newtonsoft.Json;
using System.Text;

namespace WebApp.Services;

public class MongoTurboService
{
    private readonly HttpClient _httpClient = new();
    private readonly string _baseUrl = "http://localhost:6060";

    /// <summary>
    /// Sets a cache value using the Rust service.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <param name="value">The cache value.</param>
    /// <param name="ttl">Optional TTL in seconds (default is 60 seconds).</param>
    /// <returns>The response string from the Rust service.</returns>
    public async Task<string> SetAsync(string key, string value, int ttl = 60)
    {
        var payload = new { key, value, ttl };
        var jsonPayload = JsonConvert.SerializeObject(payload);
        var content = new StringContent(jsonPayload, Encoding.UTF8, "application/json");

        var response = await _httpClient.PostAsync($"{_baseUrl}/set", content);
        response.EnsureSuccessStatusCode();

        return await response.Content.ReadAsStringAsync();
    }

    /// <summary>
    /// Gets a cache value using the Rust service.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <returns>The response string (JSON) from the Rust service.</returns>
    public async Task<string> GetAsync(string key)
    {
        var response = await _httpClient.GetAsync($"{_baseUrl}/get?key={Uri.EscapeDataString(key)}");
        response.EnsureSuccessStatusCode();

        return await response.Content.ReadAsStringAsync();
    }
}
using Microsoft.Extensions.Caching.Distributed;

namespace WebApp.Services;

public class RedisService
{
    private readonly IDistributedCache _cache;

    public RedisService(IDistributedCache cache)
    {
        _cache = cache;
    }

    public async Task<string?> GetAsync(string key)
        => await _cache.GetStringAsync(key);

    public async Task SetAsync(string key, string value)
        => await _cache.SetStringAsync(key, value, new DistributedCacheEntryOptions()
        {
            AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1),
            SlidingExpiration = TimeSpan.FromMinutes(1)
        });
}
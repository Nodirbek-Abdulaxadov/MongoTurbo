using Microsoft.AspNetCore.Mvc;
using Newtonsoft.Json;
using System.Diagnostics;
using WebApp.Services;

namespace WebApp.Controllers;

[ApiController]
[Route("[controller]")]
public class HighLoadTestController(RedisService redisService,
                                    RustCacheClient rustCacheService)
    : ControllerBase
{
    // This endpoint tests the "set" operations under high load.
    [HttpGet("set-highload")]
    public async Task<IActionResult> SetHighloadTest()
    {
        // Assume Get() returns the data to be cached.
        var items = JsonConvert.SerializeObject(Get());
        int iterations = 10_000;
        var redisTimes = new List<double>();
        var mongoTimes = new List<double>();

        // Test Redis set operations
        for (int i = 0; i < iterations; i++)
        {
            Stopwatch sw = Stopwatch.StartNew();
            await redisService.SetAsync("weathers", items);
            sw.Stop();
            redisTimes.Add(sw.Elapsed.TotalMilliseconds);
        }

        // Test RustCache set operations
        for (int i = 0; i < iterations; i++)
        {
            Stopwatch sw = Stopwatch.StartNew();
            await rustCacheService.SetAsync("weathers", items);
            sw.Stop();
            mongoTimes.Add(sw.Elapsed.TotalMilliseconds);
        }

        var result = new
        {
            RedisSet = new
            {
                Min = redisTimes.Min(),
                Max = redisTimes.Max(),
                Average = redisTimes.Average()
            },
            RustCacheSet = new
            {
                Min = mongoTimes.Min(),
                Max = mongoTimes.Max(),
                Average = mongoTimes.Average()
            }
        };

        return Ok(result);
    }

    // This endpoint tests the "get" operations under high load.
    [HttpGet("get-highload")]
    public async Task<IActionResult> GetHighloadTest()
    {
        int iterations = 10_000;
        var redisTimes = new List<double>();
        var mongoTimes = new List<double>();

        // Test Redis get operations
        for (int i = 0; i < iterations; i++)
        {
            Stopwatch sw = Stopwatch.StartNew();
            await redisService.GetAsync("weathers");
            sw.Stop();
            redisTimes.Add(sw.Elapsed.TotalMilliseconds);
        }

        // Test RustCache get operations
        for (int i = 0; i < iterations; i++)
        {
            Stopwatch sw = Stopwatch.StartNew();
            await rustCacheService.GetAsync("weathers");
            sw.Stop();
            mongoTimes.Add(sw.Elapsed.TotalMilliseconds);
        }

        var result = new
        {
            RedisGet = new
            {
                Min = redisTimes.Min(),
                Max = redisTimes.Max(),
                Average = redisTimes.Average()
            },
            RustCacheGet = new
            {
                Min = mongoTimes.Min(),
                Max = mongoTimes.Max(),
                Average = mongoTimes.Average()
            }
        };

        return Ok(result);
    }

    // This endpoint tests high-load parallel "set" operations.
    [HttpGet("set-highload-parallel")]
    public async Task<IActionResult> SetHighloadParallelTest()
    {
        var items = Newtonsoft.Json.JsonConvert.SerializeObject(Get());
        int iterations = 1_000;

        // Parallel test for Redis
        var redisTasks = new List<Task<double>>();
        for (int i = 0; i < iterations; i++)
        {
            redisTasks.Add(MeasureExecutionTimeAsync(() => redisService.SetAsync("weathers", items)));
        }

        // Parallel test for RustCache
        var mongoTasks = new List<Task<double>>();
        for (int i = 0; i < iterations; i++)
        {
            mongoTasks.Add(MeasureExecutionTimeAsync(() => rustCacheService.SetAsync("weathers", items)));
        }

        // Run both tests concurrently
        var results = await Task.WhenAll(Task.WhenAll(redisTasks), Task.WhenAll(mongoTasks));

        var redisTimes = results[0];  // First Task.WhenAll result
        var mongoTimes = results[1];  // Second Task.WhenAll result

        var result = new
        {
            RedisSet = new
            {
                Min = redisTimes.Min(),
                Max = redisTimes.Max(),
                Average = redisTimes.Average()
            },
            RustCacheSet = new
            {
                Min = mongoTimes.Min(),
                Max = mongoTimes.Max(),
                Average = mongoTimes.Average()
            }
        };

        return Ok(result);
    }

    // This endpoint tests high-load parallel "get" operations.
    [HttpGet("get-highload-parallel")]
    public async Task<IActionResult> GetHighloadParallelTest()
    {
        int iterations = 1_000;

        // Parallel test for Redis
        var redisTasks = new List<Task<double>>();
        for (int i = 0; i < iterations; i++)
        {
            redisTasks.Add(MeasureExecutionTimeAsync(() => redisService.GetAsync("weathers")));
        }

        // Parallel test for RustCache
        var mongoTasks = new List<Task<double>>();
        for (int i = 0; i < iterations; i++)
        {
            mongoTasks.Add(MeasureExecutionTimeAsync(() => rustCacheService.GetAsync("weathers")));
        }

        // Run both tests concurrently
        var results = await Task.WhenAll(Task.WhenAll(redisTasks), Task.WhenAll(mongoTasks));

        var redisTimes = results[0];  // First Task.WhenAll result
        var mongoTimes = results[1];  // Second Task.WhenAll result

        var result = new
        {
            RedisSet = new
            {
                Min = redisTimes.Min(),
                Max = redisTimes.Max(),
                Average = redisTimes.Average()
            },
            RustCacheSet = new
            {
                Min = mongoTimes.Min(),
                Max = mongoTimes.Max(),
                Average = mongoTimes.Average()
            }
        };

        return Ok(result);
    }



    // Measure execution time of an asynchronous function
    private async Task<double> MeasureExecutionTimeAsync(Func<Task> operation)
    {
        Stopwatch sw = Stopwatch.StartNew();
        await operation();
        sw.Stop();
        return sw.Elapsed.TotalMilliseconds;
    }

    // Placeholder for data generation; replace with your actual implementation.
    private object Get()
    {
        // Generate some sample data
        return new { Temperature = 25, Humidity = 60, Condition = "Sunny" };
    }
}
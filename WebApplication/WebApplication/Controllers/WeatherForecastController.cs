using Microsoft.AspNetCore.Mvc;
using Newtonsoft.Json;
using System.Diagnostics;
using WebApp.Services;

namespace WebApp.Controllers;

[ApiController]
[Route("[controller]")]
public class WeatherForecastController(RedisService redisService, 
                                       MongoTurboService mongoTurboService) 
    : ControllerBase
{
    private static readonly string[] Summaries =
    [
        "Freezing", "Bracing", "Chilly", "Cool", "Mild", "Warm", "Balmy", "Hot", "Sweltering", "Scorching"
    ];

    [HttpGet(Name = "GetWeatherForecast")]
    public IEnumerable<WeatherForecast> Get()
    {
        return [.. Enumerable.Range(1, 5).Select(index => new WeatherForecast
        {
            Date = DateOnly.FromDateTime(DateTime.Now.AddDays(index)),
            TemperatureC = Random.Shared.Next(-20, 55),
            Summary = Summaries[Random.Shared.Next(Summaries.Length)]
        })];
    }

    [HttpGet("set")]
    public async Task<IActionResult> SetTest()
    {
        var items = JsonConvert.SerializeObject(Get());
        Stopwatch stopwatch = Stopwatch.StartNew();
        await redisService.SetAsync("weathers", items);
        stopwatch.Stop();

        Stopwatch stopwatch1 = Stopwatch.StartNew();
        await mongoTurboService.SetAsync("weathers", items);
        stopwatch1.Stop();

        var result = $"""
        Redis set time:         {stopwatch.Elapsed.TotalMilliseconds} ms
        MongoTurbo set time:    {stopwatch1.Elapsed.TotalMilliseconds} ms
        """;

        return Ok(result);
    }

    [HttpGet("get")]
    public async Task<IActionResult> GetTest()
    {
        Stopwatch stopwatch = Stopwatch.StartNew();
        var items1 = await redisService.GetAsync("weathers");
        stopwatch.Stop();

        Stopwatch stopwatch1 = Stopwatch.StartNew();
        var items2 = await mongoTurboService.GetAsync("weathers");
        stopwatch1.Stop();

        var result = $"""
        Redis get time:         {stopwatch.Elapsed.TotalMilliseconds} ms
        MongoTurbo get time:    {stopwatch1.Elapsed.TotalMilliseconds} ms
        """;

        return Ok(result);
    }
}

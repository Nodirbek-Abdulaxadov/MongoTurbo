using System;
using System.Net.Sockets;
using System.Text;
using System.Threading.Tasks;

public class RustCacheClient : IDisposable
{
    private readonly string _host;
    private readonly int _port;
    private TcpClient _client;
    private NetworkStream _stream;

    public RustCacheClient(string host = "localhost", int port = 6060)
    {
        _host = host;
        _port = port;
    }

    private async Task EnsureConnectedAsync()
    {
        if (_client == null || !_client.Connected)
        {
            _client = new TcpClient();
            await _client.ConnectAsync(_host, _port);
            _stream = _client.GetStream();
        }
    }

    private async Task<string> SendCommandAsync(string command)
    {
        await EnsureConnectedAsync();

        var data = Encoding.UTF8.GetBytes(command + "\n");
        await _stream.WriteAsync(data, 0, data.Length);
        await _stream.FlushAsync();

        var buffer = new byte[1024];
        int bytesRead = await _stream.ReadAsync(buffer, 0, buffer.Length);
        return Encoding.UTF8.GetString(buffer, 0, bytesRead).Trim();
    }

    public async Task<string> GetAsync(string key) =>
        await SendCommandAsync($"GET {key}");

    public async Task SetAsync(string key, string value) =>
        await SendCommandAsync($"SET {key} {value}");

    public void Dispose()
    {
        _stream?.Dispose();
        _client?.Dispose();
    }
}

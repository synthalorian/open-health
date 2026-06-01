import 'dart:async';
import 'dart:convert';
import 'dart:io';

/// Unix domain socket client for communicating with the open_health Rust server.
///
/// Protocol: newline-delimited JSON (NDJSON). Each request is a single JSON
/// object on one line; each response is a single JSON object on one line.
class IpcClient {
  final String socketPath;
  Socket? _socket;
  final _responseController = StreamController<Map<String, dynamic>>.broadcast();
  final _buffer = StringBuffer();
  bool _connecting = false;

  IpcClient({this.socketPath = '/tmp/open_health.sock'});

  /// Attempt to connect to the Unix socket. Returns true on success.
  Future<bool> connect() async {
    if (_socket != null) return true;
    if (_connecting) {
      // Wait briefly for in-flight connection
      await Future.delayed(const Duration(milliseconds: 200));
      return _socket != null;
    }
    _connecting = true;
    try {
      _socket = await Socket.connect(
        InternetAddress(socketPath, type: InternetAddressType.unix),
        0,
        timeout: const Duration(seconds: 2),
      );
      _socket!.listen(
        _onData,
        onError: (e) => disconnect(),
        onDone: disconnect,
      );
      return true;
    } catch (_) {
      return false;
    } finally {
      _connecting = false;
    }
  }

  bool get isConnected => _socket != null;

  void _onData(List<int> data) {
    _buffer.write(utf8.decode(data));
    // NDJSON: split on newlines
    while (true) {
      final raw = _buffer.toString();
      final idx = raw.indexOf('\n');
      if (idx == -1) break;
      final line = raw.substring(0, idx).trim();
      _buffer.clear();
      if (idx + 1 < raw.length) {
        _buffer.write(raw.substring(idx + 1));
      }
      if (line.isEmpty) continue;
      try {
        final decoded = jsonDecode(line) as Map<String, dynamic>;
        _responseController.add(decoded);
      } catch (_) {
        // Malformed line, ignore
      }
    }
  }

  /// Send a request and wait for the next response.
  Future<Map<String, dynamic>> send(Map<String, dynamic> request) async {
    if (!isConnected) {
      final ok = await connect();
      if (!ok) return _mockResponse(request);
    }

    final payload = jsonEncode(request);
    _socket!.write('$payload\n');
    await _socket!.flush();

    // Wait for next response with timeout
    try {
      final resp = await _responseController.stream.first.timeout(
        const Duration(seconds: 5),
      );
      return resp;
    } on TimeoutException {
      return {'error': 'Request timed out'};
    }
  }

  Map<String, dynamic> _mockResponse(Map<String, dynamic> request) {
    final method = request['method'] as String? ?? '';
    switch (method) {
      case 'ping':
        return {'status': 'pong'};
      case 'get_dashboard':
        return {
          'sleep_hours': 7.5,
          'hrv': 42.0,
          'resting_hr': 58,
          'steps': 8432,
          'calories': 2150.0,
          'weight_kg': 78.5,
        };
      case 'get_records':
        return {'records': []};
      case 'list_imports':
        return {'sessions': []};
      default:
        return {'status': 'ok'};
    }
  }

  void disconnect() {
    _socket?.destroy();
    _socket = null;
  }
}

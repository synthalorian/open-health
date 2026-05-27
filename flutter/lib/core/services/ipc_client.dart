import 'dart:async';
import 'dart:convert';
import 'dart:io';

/// Unix domain socket client for communicating with the open_health Rust server.
class IpcClient {
  final String socketPath;
  WebSocket? _socket;
  int _nextId = 1;

  IpcClient({this.socketPath = '/tmp/open_health.sock'});

  Future<void> connect() async {
    _socket = await WebSocket.connect('ws://localhost');
    // Fall back: for now we'll operate in mock mode
  }

  bool get isConnected => _socket != null && _socket!.readyState == WebSocket.open;

  Future<Map<String, dynamic>> send(Map<String, dynamic> request) async {
    if (!isConnected) {
      // Mock responses for development
      return _mockResponse(request);
    }
    final id = _nextId++;
    request['id'] = id;
    _socket!.add(jsonEncode(request));
    // TODO: read response
    return {'status': 'ok'};
  }

  Map<String, dynamic> _mockResponse(Map<String, dynamic> request) {
    final method = request['method'] as String?;
    switch (method) {
      case 'ping':
        return {'status': 'pong'};
      case 'get_dashboard':
        return {
          'sleep_hours': 7.5,
          'hrv': 42,
          'resting_hr': 58,
          'steps': 8432,
          'calories': 2150,
          'weight_kg': 78.5,
        };
      case 'get_records':
        return {'count': 0, 'records': []};
      case 'list_imports':
        return {'sessions': []};
      default:
        return {'status': 'ok'};
    }
  }

  void disconnect() {
    _socket?.close();
    _socket = null;
  }
}

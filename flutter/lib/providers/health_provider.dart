import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../core/services/ipc_client.dart';

// ─── Client Provider ────────────────────────────────────────────────────

final ipcClientProvider = Provider<IpcClient>((ref) {
  final client = IpcClient();
  ref.onDispose(() => client.disconnect());
  return client;
});

// ─── Dashboard Data ─────────────────────────────────────────────────────

class DashboardData {
  final double sleepHours;
  final double hrv;
  final int restingHr;
  final int steps;
  final double calories;
  final double weightKg;

  const DashboardData({
    this.sleepHours = 0,
    this.hrv = 0,
    this.restingHr = 0,
    this.steps = 0,
    this.calories = 0,
    this.weightKg = 0,
  });
}

final dashboardDataProvider = FutureProvider.autoDispose<DashboardData>((ref) async {
  final client = ref.watch(ipcClientProvider);
  final resp = await client.send({'method': 'get_dashboard'});
  if (resp['error'] != null) throw Exception(resp['error']);
  return DashboardData(
    sleepHours: (resp['sleep_hours'] as num?)?.toDouble() ?? 0,
    hrv: (resp['hrv'] as num?)?.toDouble() ?? 0,
    restingHr: (resp['resting_hr'] as int?) ?? 0,
    steps: (resp['steps'] as int?) ?? 0,
    calories: (resp['calories'] as num?)?.toDouble() ?? 0,
    weightKg: (resp['weight_kg'] as num?)?.toDouble() ?? 0,
  );
});

// ─── Database Status ────────────────────────────────────────────────────

final dbStatusProvider = FutureProvider.autoDispose<String>((ref) async {
  final client = ref.watch(ipcClientProvider);
  final resp = await client.send({'method': 'ping'});
  return resp['status'] as String? ?? 'unknown';
});

// ─── Health Records ─────────────────────────────────────────────────────

class HealthRecord {
  final String id;
  final String recordType;
  final DateTime timestamp;
  final double value;
  final String unit;
  final String? source;

  HealthRecord({
    required this.id,
    required this.recordType,
    required this.timestamp,
    required this.value,
    required this.unit,
    this.source,
  });

  factory HealthRecord.fromJson(Map<String, dynamic> json) {
    return HealthRecord(
      id: json['id'] as String? ?? '',
      recordType: json['record_type'] as String? ?? '',
      timestamp: DateTime.tryParse(json['timestamp'] as String? ?? '') ?? DateTime.now(),
      value: (json['value'] as num?)?.toDouble() ?? 0,
      unit: json['unit'] as String? ?? '',
      source: json['source'] as String?,
    );
  }
}

final heartRateRecordsProvider = FutureProvider.autoDispose<List<HealthRecord>>((ref) async {
  final client = ref.watch(ipcClientProvider);
  final now = DateTime.now();
  final from = now.subtract(const Duration(days: 7));
  final resp = await client.send({
    'method': 'get_records',
    'record_type': 'HeartRate',
    'from': from.toIso8601String(),
    'to': now.toIso8601String(),
  });
  if (resp['error'] != null) throw Exception(resp['error']);
  final list = resp['records'] as List<dynamic>? ?? [];
  return list.map((e) => HealthRecord.fromJson(e as Map<String, dynamic>)).toList();
});

final sleepRecordsProvider = FutureProvider.autoDispose<List<HealthRecord>>((ref) async {
  final client = ref.watch(ipcClientProvider);
  final now = DateTime.now();
  final from = now.subtract(const Duration(days: 7));
  final resp = await client.send({
    'method': 'get_sleep_records',
    'from': from.toIso8601String().split('T').first,
    'to': now.toIso8601String().split('T').first,
  });
  if (resp['error'] != null) throw Exception(resp['error']);
  final list = resp['records'] as List<dynamic>? ?? [];
  return list.map((e) => HealthRecord.fromJson(e as Map<String, dynamic>)).toList();
});

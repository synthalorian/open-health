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

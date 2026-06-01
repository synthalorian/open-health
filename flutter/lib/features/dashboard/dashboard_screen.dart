import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fl_chart/fl_chart.dart';
import '../../providers/health_provider.dart';

class DashboardScreen extends ConsumerWidget {
  const DashboardScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final asyncData = ref.watch(dashboardDataProvider);
    final hrAsync = ref.watch(heartRateRecordsProvider);
    final sleepAsync = ref.watch(sleepRecordsProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Open Health'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () {
              ref.invalidate(dashboardDataProvider);
              ref.invalidate(heartRateRecordsProvider);
              ref.invalidate(sleepRecordsProvider);
            },
          ),
        ],
      ),
      body: asyncData.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, _) => Center(
          child: Padding(
            padding: const EdgeInsets.all(32),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(Icons.cloud_off, size: 64, color: theme.hintColor),
                const SizedBox(height: 16),
                Text('Could not load health data',
                    style: theme.textTheme.titleMedium),
                const SizedBox(height: 8),
                Text('$err', style: TextStyle(color: theme.hintColor)),
                const SizedBox(height: 16),
                FilledButton(
                  onPressed: () => ref.invalidate(dashboardDataProvider),
                  child: const Text('Retry'),
                ),
              ],
            ),
          ),
        ),
        data: (data) => RefreshIndicator(
          onRefresh: () async {
            ref.invalidate(dashboardDataProvider);
            ref.invalidate(heartRateRecordsProvider);
            ref.invalidate(sleepRecordsProvider);
          },
          child: SingleChildScrollView(
            physics: const AlwaysScrollableScrollPhysics(),
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Overview cards row
                Row(
                  children: [
                    Expanded(
                      child: _MetricCard(
                        icon: Icons.bedtime,
                        label: 'Sleep',
                        value: '${data.sleepHours.toStringAsFixed(1)}h',
                        color: const Color(0xFF4ECDC4),
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: _MetricCard(
                        icon: Icons.favorite,
                        label: 'HRV',
                        value: '${data.hrv.toStringAsFixed(0)} ms',
                        color: const Color(0xFFFF6B6B),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 12),
                Row(
                  children: [
                    Expanded(
                      child: _MetricCard(
                        icon: Icons.monitor_heart,
                        label: 'Resting HR',
                        value: '${data.restingHr} bpm',
                        color: const Color(0xFFFFD93D),
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: _MetricCard(
                        icon: Icons.directions_walk,
                        label: 'Steps',
                        value: '${data.steps}',
                        color: const Color(0xFF6C63FF),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 12),
                Row(
                  children: [
                    Expanded(
                      child: _MetricCard(
                        icon: Icons.local_fire_department,
                        label: 'Calories',
                        value: '${data.calories.toStringAsFixed(0)}',
                        color: const Color(0xFFFF6B6B),
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: _MetricCard(
                        icon: Icons.monitor_weight,
                        label: 'Weight',
                        value: '${data.weightKg.toStringAsFixed(1)} kg',
                        color: const Color(0xFF4ECDC4),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 24),

                // Heart Rate Chart
                Text('Heart Rate (Last 7 Days)',
                    style: theme.textTheme.titleMedium
                        ?.copyWith(fontWeight: FontWeight.bold)),
                const SizedBox(height: 12),
                SizedBox(
                  height: 200,
                  child: hrAsync.when(
                    loading: () => const Center(child: CircularProgressIndicator()),
                    error: (_, __) => const _PlaceholderChart(),
                    data: (records) => _HeartRateChart(records: records),
                  ),
                ),
                const SizedBox(height: 24),

                // Sleep chart
                Text('Sleep Duration (Last 7 Days)',
                    style: theme.textTheme.titleMedium
                        ?.copyWith(fontWeight: FontWeight.bold)),
                const SizedBox(height: 12),
                SizedBox(
                  height: 200,
                  child: sleepAsync.when(
                    loading: () => const Center(child: CircularProgressIndicator()),
                    error: (_, __) => const _PlaceholderChart(),
                    data: (records) => _SleepChart(records: records),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _MetricCard extends StatelessWidget {
  final IconData icon;
  final String label;
  final String value;
  final Color color;

  const _MetricCard({
    required this.icon,
    required this.label,
    required this.value,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Icon(icon, color: color, size: 28),
            const SizedBox(height: 8),
            Text(value,
                style: theme.textTheme.headlineSmall?.copyWith(
                    fontWeight: FontWeight.bold, color: color)),
            Text(label, style: TextStyle(color: theme.hintColor)),
          ],
        ),
      ),
    );
  }
}

class _PlaceholderChart extends StatelessWidget {
  const _PlaceholderChart();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Text(
        'No data available',
        style: TextStyle(color: Theme.of(context).hintColor),
      ),
    );
  }
}

class _HeartRateChart extends StatelessWidget {
  final List<HealthRecord> records;
  const _HeartRateChart({required this.records});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    if (records.isEmpty) return const _PlaceholderChart();

    final spots = records.asMap().entries.map((e) {
      return FlSpot(e.key.toDouble(), e.value.value);
    }).toList();

    return LineChart(
      LineChartData(
        gridData: FlGridData(
          show: true,
          drawVerticalLine: false,
          getDrawingHorizontalLine: (value) => FlLine(
            color: theme.dividerColor.withValues(alpha: 0.2),
            strokeWidth: 1,
          ),
        ),
        titlesData: const FlTitlesData(
          leftTitles: AxisTitles(sideTitles: SideTitles(showTitles: false)),
          bottomTitles: AxisTitles(sideTitles: SideTitles(showTitles: false)),
          topTitles: AxisTitles(sideTitles: SideTitles(showTitles: false)),
          rightTitles: AxisTitles(sideTitles: SideTitles(showTitles: false)),
        ),
        borderData: FlBorderData(show: false),
        lineBarsData: [
          LineChartBarData(
            spots: spots,
            isCurved: true,
            color: const Color(0xFFFF6B6B),
            barWidth: 3,
            dotData: const FlDotData(show: false),
            belowBarData: BarAreaData(
              show: true,
              color: const Color(0xFFFF6B6B).withValues(alpha: 0.1),
            ),
          ),
        ],
      ),
    );
  }
}

class _SleepChart extends StatelessWidget {
  final List<HealthRecord> records;
  const _SleepChart({required this.records});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    if (records.isEmpty) return const _PlaceholderChart();

    return BarChart(
      BarChartData(
        gridData: FlGridData(
          show: true,
          drawVerticalLine: false,
          getDrawingHorizontalLine: (value) => FlLine(
            color: theme.dividerColor.withValues(alpha: 0.2),
            strokeWidth: 1,
          ),
        ),
        titlesData: const FlTitlesData(
          leftTitles: AxisTitles(sideTitles: SideTitles(showTitles: false)),
          bottomTitles: AxisTitles(sideTitles: SideTitles(showTitles: false)),
          topTitles: AxisTitles(sideTitles: SideTitles(showTitles: false)),
          rightTitles: AxisTitles(sideTitles: SideTitles(showTitles: false)),
        ),
        borderData: FlBorderData(show: false),
        barGroups: records.asMap().entries.map((e) {
          return BarChartGroupData(x: e.key, barRods: [
            BarChartRodData(
              toY: e.value.value,
              color: const Color(0xFF4ECDC4),
              width: 12,
              borderRadius: const BorderRadius.vertical(top: Radius.circular(4)),
            ),
          ]);
        }).toList(),
      ),
    );
  }
}

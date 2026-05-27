import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class ReportsScreen extends ConsumerWidget {
  const ReportsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Reports'),
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: () => _showGenerateDialog(context),
            tooltip: 'Generate Report',
          ),
        ],
      ),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(Icons.assessment_outlined,
                  size: 80, color: theme.hintColor),
              const SizedBox(height: 16),
              Text('No Reports Yet',
                  style: theme.textTheme.titleMedium
                      ?.copyWith(fontWeight: FontWeight.bold)),
              const SizedBox(height: 8),
              Text(
                'Generate weekly or monthly PDF reports\n'
                'to track your health trends over time.',
                textAlign: TextAlign.center,
                style: TextStyle(color: theme.hintColor, height: 1.5),
              ),
              const SizedBox(height: 24),
              FilledButton.icon(
                onPressed: () => _showGenerateDialog(context),
                icon: const Icon(Icons.add),
                label: const Text('Generate Report'),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _showGenerateDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Generate Report'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Select report type:'),
            const SizedBox(height: 16),
            ListTile(
              leading: const Icon(Icons.calendar_view_week),
              title: const Text('Weekly Summary'),
              subtitle: const Text('Last 7 days'),
              onTap: () {
                Navigator.pop(ctx);
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                      content: Text('Report generation coming soon')),
                );
              },
            ),
            ListTile(
              leading: const Icon(Icons.calendar_month),
              title: const Text('Monthly Summary'),
              subtitle: const Text('Last 30 days'),
              onTap: () {
                Navigator.pop(ctx);
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                      content: Text('Report generation coming soon')),
                );
              },
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
        ],
      ),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class ImportsScreen extends ConsumerWidget {
  const ImportsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Import Data'),
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: () => _showImportDialog(context),
          ),
        ],
      ),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(Icons.file_upload_outlined,
                  size: 80, color: theme.hintColor),
              const SizedBox(height: 16),
              Text('No Imports Yet',
                  style: theme.textTheme.titleMedium
                      ?.copyWith(fontWeight: FontWeight.bold)),
              const SizedBox(height: 8),
              Text(
                'Import CSV or JSON exports from your\n'
                'health trackers to populate your dashboard.',
                textAlign: TextAlign.center,
                style: TextStyle(color: theme.hintColor, height: 1.5),
              ),
              const SizedBox(height: 24),
              FilledButton.icon(
                onPressed: () => _showImportDialog(context),
                icon: const Icon(Icons.add),
                label: const Text('Import File'),
              ),
              const SizedBox(height: 12),
              Text(
                'Supported: Fitbit, Oura, Apple Health,\nGarmin, Whoop, Generic CSV',
                textAlign: TextAlign.center,
                style: TextStyle(
                    color: theme.hintColor,
                    fontSize: 12,
                    height: 1.5),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _showImportDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Import Health Data'),
        content: const Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('Select a file to import.'),
            SizedBox(height: 16),
            // TODO: Add file picker integration
            Text('File picker coming soon...',
                style: TextStyle(color: Colors.grey)),
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

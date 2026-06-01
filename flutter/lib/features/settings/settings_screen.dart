import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/theme/app_theme.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final currentMode = ref.watch(themeModeProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // Theme
          Text('Appearance',
              style: theme.textTheme.titleMedium
                  ?.copyWith(fontWeight: FontWeight.bold)),
          const SizedBox(height: 8),
          Card(
            child: Column(
              children: [
                ListTile(
                  leading: const Icon(Icons.dark_mode),
                  title: const Text('Dark Mode (Synthwave)'),
                  subtitle: const Text('Neon gradients, dark vibes'),
                  trailing: currentMode == ThemeMode.dark
                      ? const Icon(Icons.check_circle, color: Colors.green)
                      : null,
                  onTap: () => ref.read(themeModeProvider.notifier).setMode(ThemeMode.dark),
                ),
                const Divider(height: 1),
                ListTile(
                  leading: const Icon(Icons.light_mode),
                  title: const Text('Light Mode'),
                  subtitle: const Text('Clean, minimal interface'),
                  trailing: currentMode == ThemeMode.light
                      ? const Icon(Icons.check_circle, color: Colors.green)
                      : null,
                  onTap: () => ref.read(themeModeProvider.notifier).setMode(ThemeMode.light),
                ),
              ],
            ),
          ),
          const SizedBox(height: 24),

          // Database
          Text('Database',
              style: theme.textTheme.titleMedium
                  ?.copyWith(fontWeight: FontWeight.bold)),
          const SizedBox(height: 8),
          Card(
            child: Column(
              children: [
                ListTile(
                  leading: const Icon(Icons.storage),
                  title: const Text('Database Status'),
                  subtitle: const Text('Connected'),
                  trailing: const Icon(Icons.check_circle, color: Colors.green),
                ),
                const Divider(height: 1),
                ListTile(
                  leading: const Icon(Icons.lock),
                  title: const Text('Encryption'),
                  subtitle: const Text('AES-GCM-256'),
                  trailing: const Icon(Icons.check_circle, color: Colors.green),
                ),
              ],
            ),
          ),
          const SizedBox(height: 24),

          // About
          Text('About',
              style: theme.textTheme.titleMedium
                  ?.copyWith(fontWeight: FontWeight.bold)),
          const SizedBox(height: 8),
          Card(
            child: Column(
              children: [
                ListTile(
                  leading: const Icon(Icons.info_outline),
                  title: const Text('Open Health'),
                  subtitle: const Text('Version 0.1.0'),
                ),
                const Divider(height: 1),
                ListTile(
                  leading: const Icon(Icons.code),
                  title: const Text('Built with'),
                  subtitle: const Text('Rust + Flutter'),
                ),
                const Divider(height: 1),
                ListTile(
                  leading: const Icon(Icons.privacy_tip_outlined),
                  title: const Text('Privacy'),
                  subtitle: const Text(
                      '100% local. No data ever leaves your device.'),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

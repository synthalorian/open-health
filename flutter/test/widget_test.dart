import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:open_health/app.dart';

void main() {
  testWidgets('App renders with navigation', (WidgetTester tester) async {
    await tester.pumpWidget(
      const ProviderScope(
        child: OpenHealthApp(),
      ),
    );

    // Wait for async initialization
    await tester.pumpAndSettle();

    // Should find the app title in the dashboard app bar
    expect(find.text('Open Health'), findsOneWidget);

    // Should have bottom navigation
    expect(find.byType(NavigationBar), findsOneWidget);

    // Should have dashboard tab selected by default
    expect(find.text('Dashboard'), findsOneWidget);
  });

  testWidgets('Can navigate to Settings tab', (WidgetTester tester) async {
    await tester.pumpWidget(
      const ProviderScope(
        child: OpenHealthApp(),
      ),
    );
    await tester.pumpAndSettle();

    // Tap on Settings tab
    await tester.tap(find.text('Settings'));
    await tester.pumpAndSettle();

    // Should show settings content
    expect(find.text('Appearance'), findsOneWidget);
    expect(find.text('Database'), findsOneWidget);
    expect(find.text('About'), findsOneWidget);
  });
}

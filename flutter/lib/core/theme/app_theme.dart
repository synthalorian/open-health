import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

// ─── Theme Mode Provider ────────────────────────────────────────────────

final themeModeProvider = NotifierProvider<ThemeModeNotifier, ThemeMode>(
  () => ThemeModeNotifier(),
);

class ThemeModeNotifier extends Notifier<ThemeMode> {
  static const _key = 'theme_mode';

  @override
  ThemeMode build() {
    _load();
    return ThemeMode.dark;
  }

  Future<void> _load() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final value = prefs.getString(_key);
      if (value == 'light') state = ThemeMode.light;
      if (value == 'dark') state = ThemeMode.dark;
    } catch (_) {}
  }

  Future<void> setMode(ThemeMode mode) async {
    state = mode;
    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(_key, mode.name);
    } catch (_) {}
  }
}

// ─── App Theme ──────────────────────────────────────────────────────────

class AppTheme {
  AppTheme._();

  // Synthwave neon palette
  static const _electricPurple = Color(0xFF8F00FF);
  static const _hotPink = Color(0xFFFF7EDB);
  static const _neonYellow = Color(0xFFF3E70F);
  static const _darkBg = Color(0xFF0A0014);
  static const _cardBg = Color(0xFF1A0033);
  static const _navBg = Color(0xFF140026);

  static final dark = ThemeData(
    brightness: Brightness.dark,
    useMaterial3: true,
    colorScheme: const ColorScheme.dark(
      primary: _electricPurple,
      secondary: _hotPink,
      tertiary: _neonYellow,
      surface: _cardBg,
      onPrimary: Colors.white,
      onSecondary: Colors.white,
      onTertiary: Colors.black,
    ),
    scaffoldBackgroundColor: _darkBg,
    appBarTheme: const AppBarTheme(
      backgroundColor: _cardBg,
      foregroundColor: _hotPink,
      elevation: 0,
    ),
    cardTheme: CardThemeData(
      color: _cardBg.withValues(alpha: 0.9),
      elevation: 4,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
        side: const BorderSide(
          color: _electricPurple,
          width: 1,
        ),
      ),
    ),
    navigationBarTheme: const NavigationBarThemeData(
      backgroundColor: _navBg,
      indicatorColor: _electricPurple,
      indicatorShape: RoundedRectangleBorder(
        borderRadius: BorderRadius.all(Radius.circular(8)),
      ),
    ),
    textTheme: const TextTheme(
      headlineSmall: TextStyle(color: Colors.white, fontWeight: FontWeight.bold),
      titleMedium: TextStyle(color: Colors.white),
      bodyMedium: TextStyle(color: Colors.white70),
    ),
  );

  static final light = ThemeData(
    brightness: Brightness.light,
    useMaterial3: true,
    colorScheme: const ColorScheme.light(
      primary: _electricPurple,
      secondary: _hotPink,
      tertiary: _neonYellow,
      surface: Colors.white,
    ),
    scaffoldBackgroundColor: const Color(0xFFF8FAFC),
    appBarTheme: const AppBarTheme(
      backgroundColor: Colors.white,
      foregroundColor: _electricPurple,
      elevation: 0,
    ),
    cardTheme: CardThemeData(
      color: Colors.white,
      elevation: 2,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
      ),
    ),
  );
}

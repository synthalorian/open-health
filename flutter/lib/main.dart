import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod';
import 'app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Enforce portrait mode for health dashboard
  SystemChrome.setPreferredOrientations([
    DeviceOrientation.portraitUp,
    DeviceOrientation.portraitDown,
  ]);
  
  // Load synthwave fonts
  SystemFonts.loadFonts();
  
  runApp(
    UncontrolledNotifierProvider(
      child: const OpenHealthApp(),
    ),
  );
}

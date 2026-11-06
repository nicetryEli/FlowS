import 'package:flutter/material.dart';

/// Flat, borderless slate-teal design system.
/// Background: radial gradient center #242f31 → corners #1a2224.
/// Active highlights: muted white (#ffffff) + dark slate (#141a1b) — no neon cyan.
abstract final class AppColors {
  static const Color bg = Color(0xFF16242C);             // corner colour of the Canvas gradient (dark slate blue)
  static const Color bgGradientCenter = Color(0xFF242F31);
  static const Color bgDeep = Color(0xFF1A2224);         // rail / aside / footer
  static const Color cell = Color(0xE62D393B);           // semi-transparent dark teal tile
  static const Color cell2 = Color(0xD02D393B);          // out-of-month tile (dark-blue hatch overlays it)
  static const Color cellHover = Color(0xFF323E40);
  static const Color today = Color(0xFF7195A0);          // filled Today cell, white text

  static const Color text = Color(0xFFE8ECEC);
  static const Color white = Color(0xFFFFFFFF);
  static const Color slate = Color(0xFF141A1B);          // active pill / buttons
  static const Color muted = Color(0xFF8A9799);
  static const Color muted2 = Color(0xFF5F6F71);

  // Event accent palette (thin vertical bars).
  static const Color blue = Color(0xFF03A9F4);
  static const Color green = Color(0xFF4CAF50);
  static const Color orange = Color(0xFFFF9800);
  static const Color pink = Color(0xFFE91E63);
  static const Color gold = Color(0xFFFF9800);           // personal 'yellow' maps to orange accent
  static const Color purple = Color(0xFFE91E63);         // personal 'purple' maps to pink accent
  static const Color magenta = Color(0xFFE91E63);
  static const Color red = Color(0xFFE91E63);            // exams map to the crimson accent
  static const Color absentAccent = Color(0xFF6B7A7D);

}

abstract final class AppRadii {
  static const double cell = 7;
  static const double chip = 4;
  static const double control = 6;
  static const double iconRail = 6;
}

/// Radial canvas background: center glow fading into dark corners.
const BoxDecoration appBackground = BoxDecoration(
  gradient: RadialGradient(
    center: Alignment(0.0, -0.25),
    radius: 1.15,
    colors: [AppColors.bgGradientCenter, AppColors.bg],
    stops: [0.0, 1.0],
  ),
);

ThemeData buildAppTheme() {
  const scheme = ColorScheme.dark(
    primary: AppColors.white,
    surface: AppColors.bg,
    error: AppColors.pink,
  );
  return ThemeData(
    useMaterial3: true,
    colorScheme: scheme,
    scaffoldBackgroundColor: Colors.transparent,
    splashColor: Colors.transparent,
    highlightColor: Colors.transparent,
    textSelectionTheme: const TextSelectionThemeData(cursorColor: AppColors.white),
    dialogTheme: const DialogThemeData(backgroundColor: Color(0xFF232E30)),
    fontFamily: 'Inter', // falls back to system sans if not installed
  );
}

import 'package:flutter/material.dart';

import 'screens/calendar_screen.dart';
import 'theme.dart';

void main() {
  runApp(const TdtuCalendarApp());
}

class TdtuCalendarApp extends StatelessWidget {
  const TdtuCalendarApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'TDTU · Scheduled',
      debugShowCheckedModeBanner: false,
      theme: buildAppTheme(),
      home: const CalendarScreen(),
    );
  }
}

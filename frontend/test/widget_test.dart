import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:tdtu_calendar/main.dart';

void main() {
  testWidgets('Calendar renders Scheduled tab', (WidgetTester tester) async {
    tester.view.physicalSize = const Size(1600, 900);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);

    await tester.pumpWidget(const TdtuCalendarApp());
    await tester.pumpAndSettle();

    expect(find.text('Scheduled'), findsOneWidget);
    expect(find.byIcon(Icons.calendar_month_rounded), findsOneWidget);
  });
}

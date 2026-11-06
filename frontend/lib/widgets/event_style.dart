import 'package:flutter/material.dart';

import '../models/calendar_event.dart';
import '../theme.dart';

/// Accent color per event (chips use it as a left accent bar / block tint).
Color accentFor(CalendarEvent ev) {
  switch (ev.kind) {
    case CalendarEventKind.classSession:
      switch (ev.status) {
        case SessionStatus.absent:
          return AppColors.absentAccent;
        case SessionStatus.makeup:
          return AppColors.orange;
        default:
          return AppColors.blue;
      }
    case CalendarEventKind.exam:
      return AppColors.pink;
    case CalendarEventKind.deadline:
      return AppColors.pink;
    case CalendarEventKind.personal:
      switch (ev.color) {
        case 'orange':
          return AppColors.orange;
        case 'green':
          return AppColors.green;
        case 'blue':
          return AppColors.blue;
        case 'yellow':
          return AppColors.gold;
        case 'pink':
          return AppColors.magenta;
        case 'grey':
          return AppColors.absentAccent;
        case 'purple':
          return AppColors.purple;
        default:
          return AppColors.green;
      }
  }
}

/// Soft tinted fill for week-view blocks (accent at ~14% alpha over cell).
Color tintFor(CalendarEvent ev, {double alpha = 0.16}) =>
    accentFor(ev).withValues(alpha: alpha);

String kindLabel(CalendarEvent ev) => switch (ev.kind) {
      CalendarEventKind.classSession => 'CLASS SESSION',
      CalendarEventKind.exam => 'EXAM',
      CalendarEventKind.deadline => 'ELEARNING DEADLINE',
      CalendarEventKind.personal => 'PERSONAL',
    };

const _months = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December',
];
const _weekdaysShort = ['SUN', 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'];
const _weekdaysLong = [
  'Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday',
];

String monthLabel(DateTime d) => '${_months[d.month - 1]} ${d.year}';
String weekdayShort(DateTime d) => _weekdaysShort[d.weekday % 7];
String weekdayLong(DateTime d) => _weekdaysLong[d.weekday % 7];
String shortDate(DateTime d) => '${_months[d.month - 1].substring(0, 3)} ${d.day}, ${d.year}';
String timeRange(CalendarEvent ev) {
  String hhmm(DateTime t) =>
      '${t.hour.toString().padLeft(2, '0')}:${t.minute.toString().padLeft(2, '0')}';
  if (ev.start.hour == 0 && ev.start.minute == 0 && ev.end.hour == 0 && ev.end.minute == 0) {
    return 'All day';
  }
  return '${hhmm(ev.start)} – ${hhmm(ev.end)}';
}

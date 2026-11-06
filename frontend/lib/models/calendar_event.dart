/// Unified calendar event — mirrors the SQL `calendar_events` view
/// defined in docs/erd.md: kind, ref_id, title, location, start_dt,
/// end_dt, all_day, tag, recurrence_rule.
enum CalendarEventKind { classSession, exam, deadline, personal }

enum SessionStatus { scheduled, absent, makeup, cancelled, moved }

class CalendarEvent {
  const CalendarEvent({
    required this.id,
    required this.kind,
    required this.title,
    required this.start,
    required this.end,
    this.allDay = false,
    this.location,
    this.notes,
    this.status,
    this.startPeriod,
    this.endPeriod,
    this.examType,
    this.courseName,
    this.activityUrl,
    this.color,
  });

  final String id;
  final CalendarEventKind kind;
  final String title;
  final DateTime start;
  final DateTime end;
  final bool allDay;
  final String? location;
  final String? notes;

  /// Class sessions only.
  final SessionStatus? status;
  final int? startPeriod;
  final int? endPeriod;

  /// Exams only.
  final String? examType;

  /// Deadlines only.
  final String? courseName;
  final String? activityUrl;

  /// Personal events only (hex or palette key).
  final String? color;

  /// Midnight-based day key (yyyy-mm-dd comparison done via DateUtils).
  DateTime get day => DateTime(start.year, start.month, start.day);

  bool occursOn(DateTime day) {
    final d = DateTime(day.year, day.month, day.day);
    final startDay = DateTime(start.year, start.month, start.day);
    final endDay = DateTime(end.year, end.month, end.day);
    return !d.isBefore(startDay) && !d.isAfter(endDay);
  }

  bool get isSpanning =>
      allDay && DateTime(end.year, end.month, end.day).difference(DateTime(start.year, start.month, start.day)).inDays >= 2;
}

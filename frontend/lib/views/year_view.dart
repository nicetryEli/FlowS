import 'package:flutter/material.dart';

import '../data/mock_events.dart';
import '../models/calendar_event.dart';
import '../theme.dart';

/// Year overview — 12 mini month tiles with event dots.
/// Tapping a tile's header jumps to that month in the month view.
class YearView extends StatelessWidget {
  const YearView({
    super.key,
    required this.repository,
    required this.year,
    required this.onSelectMonth,
  });

  final MockEventsRepository repository;
  final int year;
  final ValueChanged<DateTime> onSelectMonth;

  static const _monthNames = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December',
  ];

  @override
  Widget build(BuildContext context) {
    final now = DateTime.now();
    final from = DateTime(year, 1, 1);
    final to = DateTime(year, 12, 31, 23, 59);
    final events = repository.forRange(from, to);

    // Pre-index events per day with priority: exam > deadline > class > personal
    const priority = [CalendarEventKind.exam, CalendarEventKind.deadline, CalendarEventKind.classSession, CalendarEventKind.personal];
    final bestRank = <String, int>{};
    final dotByDay = <String, Color>{};
    for (final ev in events) {
      final p = priority.indexOf(ev.kind);
      final color = _dotColor(ev);
      final start = DateTime(ev.start.year, ev.start.month, ev.start.day);
      final end = DateTime(ev.end.year, ev.end.month, ev.end.day);
      for (var d = start; !d.isAfter(end); d = d.add(const Duration(days: 1))) {
        final key = '${d.year}-${d.month}-${d.day}';
        final cur = bestRank[key];
        if (cur == null || p < cur) {
          bestRank[key] = p;
          dotByDay[key] = color;
        }
      }
    }

    return Padding(
      padding: const EdgeInsets.fromLTRB(26, 12, 26, 18),
      child: SingleChildScrollView(
        child: GridView.count(
          crossAxisCount: 4,
          mainAxisSpacing: 12,
          crossAxisSpacing: 12,
          childAspectRatio: 1.05,
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          children: [
            for (var m = 1; m <= 12; m++)
              _MiniMonth(
                year: year,
                month: m,
                monthName: _monthNames[m - 1],
                dotByDay: dotByDay,
                today: now,
                onSelectMonth: onSelectMonth,
              ),
          ],
        ),
      ),
    );
  }

  static Color _dotColor(CalendarEvent ev) => switch (ev.kind) {
        CalendarEventKind.exam => AppColors.red,
        CalendarEventKind.deadline => AppColors.magenta,
        CalendarEventKind.classSession => AppColors.blue,
        CalendarEventKind.personal => AppColors.green,
      };
}

class _MiniMonth extends StatelessWidget {
  const _MiniMonth({
    required this.year,
    required this.month,
    required this.monthName,
    required this.dotByDay,
    required this.today,
    required this.onSelectMonth,
  });

  final int year;
  final int month;
  final String monthName;
  final Map<String, Color> dotByDay;
  final DateTime today;
  final ValueChanged<DateTime> onSelectMonth;

  static const _weekdayLetters = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];

  @override
  Widget build(BuildContext context) {
    final first = DateTime(year, month, 1);
    final firstDow = first.weekday % 7; // 0 = Sunday
    final daysInMonth = DateTime(year, month + 1, 0).day;

    return Material(
      color: AppColors.cell,
      borderRadius: BorderRadius.circular(AppRadii.cell),
      child: InkWell(
        onTap: () => onSelectMonth(DateTime(year, month, 1)),
        borderRadius: BorderRadius.circular(AppRadii.cell),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Text(
                    monthName,
                    style: const TextStyle(color: AppColors.text, fontSize: 13.5, fontWeight: FontWeight.w700),
                  ),
                  const Spacer(),
                  const Icon(Icons.chevron_right_rounded, size: 17, color: AppColors.muted2),
                ],
              ),
              const SizedBox(height: 8),
              Row(
                children: [
                  for (final w in _weekdayLetters)
                    Expanded(
                      child: Center(
                        child: Text(
                          w,
                          style: const TextStyle(color: AppColors.muted2, fontSize: 8, fontWeight: FontWeight.w800),
                        ),
                      ),
                    ),
                ],
              ),
              const SizedBox(height: 4),
              Expanded(
                child: GridView.builder(
                  itemCount: firstDow + daysInMonth,
                  gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                    crossAxisCount: 7,
                    childAspectRatio: 1.05,
                  ),
                  physics: const NeverScrollableScrollPhysics(),
                  itemBuilder: (context, i) {
                    if (i < firstDow) return const SizedBox.shrink();
                    final day = i - firstDow + 1;
                    final date = DateTime(year, month, day);
                    final key = '$year-$month-$day';
                    final dot = dotByDay[key];
                    final isToday = date.year == today.year && date.month == today.month && date.day == today.day;
                    return Center(
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Container(
                            width: 19,
                            height: 19,
                            alignment: Alignment.center,
                            decoration: BoxDecoration(
                              shape: BoxShape.circle,
                              color: isToday ? AppColors.white : Colors.transparent,
                            ),
                            child: Text(
                              '$day',
                              style: TextStyle(
                                color: isToday ? AppColors.slate : const Color(0xFFC9D1D1),
                                fontSize: 10.5,
                                fontWeight: isToday ? FontWeight.w800 : FontWeight.w500,
                              ),
                            ),
                          ),
                          SizedBox(
                            height: 3,
                            child: dot == null
                                ? null
                                : Container(width: 3, height: 3, decoration: BoxDecoration(color: dot, shape: BoxShape.circle)),
                          ),
                        ],
                      ),
                    );
                  },
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

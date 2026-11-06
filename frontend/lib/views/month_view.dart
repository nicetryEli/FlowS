import 'package:flutter/material.dart';

import '../data/events_repository.dart';
import '../models/calendar_event.dart';
import '../theme.dart';
import '../widgets/event_detail_dialog.dart';
import '../widgets/event_style.dart';

/// Month grid view — flat dark, striped out-of-month tiles, cyan TODAY.
/// Banners for multi-day events span the relevant week rows.
class MonthView extends StatelessWidget {
  const MonthView({
    super.key,
    required this.repository,
    required this.visibleMonth,
    required this.onQuickAdd,
  });

  final CalendarEventsRepository repository;
  final DateTime visibleMonth;
  final void Function(DateTime date) onQuickAdd;

  static const double gap = 10;
  static const List<String> weekdayHeaders = ['SUN', 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'];

  @override
  Widget build(BuildContext context) {
    final first = DateTime(visibleMonth.year, visibleMonth.month, 1);
    final firstDow = first.weekday % 7; // 0 = Sunday
    final daysInMonth = DateTime(visibleMonth.year, visibleMonth.month + 1, 0).day;
    final prevDays = DateTime(visibleMonth.year, visibleMonth.month, 0).day;
    final totalCells = (firstDow + daysInMonth);
    final weeks = (totalCells / 7).ceil();
    final now = DateTime.now();

    final from = DateTime(visibleMonth.year, visibleMonth.month, 1).subtract(const Duration(days: 7));
    final to = DateTime(visibleMonth.year, visibleMonth.month + 1, 0).add(const Duration(days: 7));
    final allEvents = repository.forRange(from, to);

    return Column(
      children: [
        // Weekday header strip
        Padding(
          padding: const EdgeInsets.fromLTRB(26, 14, 26, 8),
          child: Row(
            children: [
              for (var i = 0; i < 7; i++)
                Expanded(
                  child: Center(
                    child: Text(
                      weekdayHeaders[i],
                      style: const TextStyle(
                        color: AppColors.muted2,
                        fontSize: 10,
                        fontWeight: FontWeight.w700,
                        letterSpacing: 1.6,
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
        // Grid rows
        Expanded(
          child: LayoutBuilder(
            builder: (context, constraints) => Column(
              children: [
                for (var w = 0; w < weeks; w++)
                  Expanded(
                    child: Padding(
                      padding: EdgeInsets.only(left: 26, right: 26, bottom: w == weeks - 1 ? 18 : gap),
                      child: _WeekRow(
                        weekIndex: w,
                        firstDow: firstDow,
                        daysInMonth: daysInMonth,
                        prevDays: prevDays,
                        visibleMonth: visibleMonth,
                        repository: repository,
                        events: allEvents,
                        today: DateTime(now.year, now.month, now.day),
                        onQuickAdd: onQuickAdd,
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

class _WeekRow extends StatelessWidget {
  const _WeekRow({
    required this.weekIndex,
    required this.firstDow,
    required this.daysInMonth,
    required this.prevDays,
    required this.visibleMonth,
    required this.repository,
    required this.events,
    required this.today,
    required this.onQuickAdd,
  });

  final int weekIndex;
  final int firstDow;
  final int daysInMonth;
  final int prevDays;
  final DateTime visibleMonth;
  final CalendarEventsRepository repository;
  final List<CalendarEvent> events;
  final DateTime today;
  final void Function(DateTime date) onQuickAdd;

  @override
  Widget build(BuildContext context) {
    // Resolve the 7 cells of this row.
    final cells = <_CellInfo>[];
    for (var c = 0; c < 7; c++) {
      final index = weekIndex * 7 + c;
      final dayNum = index - firstDow + 1;
      var y = visibleMonth.year, m = visibleMonth.month, d = dayNum, inMonth = true;
      if (dayNum < 1) {
        inMonth = false; d = prevDays + dayNum; m = visibleMonth.month - 1;
        if (m < 1) { m = 12; y -= 1; }
      } else if (dayNum > daysInMonth) {
        inMonth = false; d = dayNum - daysInMonth; m = visibleMonth.month + 1;
        if (m > 12) { m = 1; y += 1; }
      }
      cells.add(_CellInfo(DateTime(y, m, d), inMonth));
    }

    final rowStart = cells.first.date;
    final rowEnd = cells.last.date;
    final banners = events.where((e) => e.isSpanning && !e.end.isBefore(rowStart) && !e.start.isAfter(rowEnd)).toList();

    return LayoutBuilder(
      builder: (context, constraints) {
        final cellW = (constraints.maxWidth - 6 * MonthView.gap) / 7;

        double bannerLeft(CalendarEvent e) {
          final s = e.start.isBefore(rowStart) ? rowStart : e.start;
          return s.difference(rowStart).inDays * (cellW + MonthView.gap);
        }

        double bannerWidth(CalendarEvent e) {
          final s = e.start.isBefore(rowStart) ? rowStart : e.start;
          final en = e.end.isAfter(DateTime(rowEnd.year, rowEnd.month, rowEnd.day, 23, 59)) ? rowEnd : e.end;
          final span = en.difference(s).inDays + 1;
          return span * cellW + (span - 1) * MonthView.gap;
        }

        return Stack(
          children: [
            Row(
              children: [
                for (var c = 0; c < 7; c++) ...[
                  if (c > 0) const SizedBox(width: MonthView.gap),
                  Expanded(child: _DayCell(info: cells[c], repository: repository, events: events, today: today, onQuickAdd: onQuickAdd)),
                ],
              ],
            ),
            for (var i = 0; i < banners.length; i++)
              Positioned(
                left: bannerLeft(banners[i]),
                width: bannerWidth(banners[i]),
                bottom: 6 + i * 26.0,
                height: 22,
                child: _SpanBanner(
                  event: banners[i],
                  roundedLeft: !banners[i].start.isBefore(rowStart),
                  roundedRight: !banners[i].end.isAfter(DateTime(rowEnd.year, rowEnd.month, rowEnd.day, 23, 59)),
                  onTap: () => showEventDetailDialog(context, banners[i], repository),
                ),
              ),
          ],
        );
      },
    );
  }
}

/// Diagonal stripes for out-of-month tiles (kept from the dark prototype).
class _StripePainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = const Color(0x590D1F2B) // dark blue @35% hatch overlay
      ..strokeWidth = 3;
    for (var x = -size.height; x < size.width + size.height; x += 12) {
      canvas.drawLine(Offset(x, size.height), Offset(x + size.height, 0), paint);
    }
  }

  @override
  bool shouldRepaint(_StripePainter oldDelegate) => false;
}

class _CellInfo {
  const _CellInfo(this.date, this.inMonth);
  final DateTime date;
  final bool inMonth;
}

class _DayCell extends StatelessWidget {
  const _DayCell({
    required this.info,
    required this.repository,
    required this.events,
    required this.today,
    required this.onQuickAdd,
  });

  final _CellInfo info;
  final CalendarEventsRepository repository;
  final List<CalendarEvent> events;
  final DateTime today;
  final void Function(DateTime date) onQuickAdd;

  @override
  Widget build(BuildContext context) {
    final inMonth = info.inMonth;
    final isToday = info.date == today;
    final dayEvents = events.where((e) => e.occursOn(info.date) && !e.isSpanning).toList()
      ..sort((a, b) => a.start.compareTo(b.start));
    final spanningCount = events.where((e) => e.isSpanning && e.occursOn(info.date)).length;

    const maxShown = 3;
    final shown = dayEvents.take(maxShown).toList();
    final hidden = dayEvents.skip(maxShown).toList();

    return Material(
      color: isToday ? AppColors.today : (inMonth ? AppColors.cell : AppColors.cell2),
      borderRadius: BorderRadius.circular(AppRadii.cell),
      child: InkWell(
        borderRadius: BorderRadius.circular(AppRadii.cell),
        onTap: () => onQuickAdd(info.date),
        child: Container(
          padding: const EdgeInsets.fromLTRB(8, 6, 10, 8),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(AppRadii.cell),
            border: inMonth ? null : Border.all(color: const Color(0x66000000), width: 1),
          ),
          child: Stack(
            children: [
              if (!inMonth)
                Positioned.fill(
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(AppRadii.cell),
                    child: CustomPaint(painter: _StripePainter()),
                  ),
                ),
              Positioned.fill(
                child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Header: day number (white on Today) + white TODAY label
              Row(
                children: [
                  Text(
                    '${info.date.day}',
                    style: TextStyle(
                      fontSize: 13,
                      fontWeight: FontWeight.w700,
                      color: isToday ? AppColors.white : (inMonth ? const Color(0xFFC9D1D1) : AppColors.muted2),
                    ),
                  ),
                  const Spacer(),
                  if (isToday)
                    const Text(
                      'TODAY',
                      style: TextStyle(color: AppColors.white, fontSize: 8.5, fontWeight: FontWeight.w800, letterSpacing: 1.4),
                    ),
                ],
              ),
              const SizedBox(height: 5),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    for (final ev in shown) _EventChip(event: ev, repository: repository),
                    if (hidden.isNotEmpty) _MoreLink(events: hidden, repository: repository),
                  ],
                ),
              ),
              // Reserve vertical space for spanning banners drawn by the row
              if (spanningCount > 0) SizedBox(height: 6 + spanningCount * 24.0),
            ],
          ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _EventChip extends StatelessWidget {
  const _EventChip({required this.event, required this.repository});

  final CalendarEvent event;
  final CalendarEventsRepository repository;

  @override
  Widget build(BuildContext context) {
    final accent = accentFor(event);
    final time = event.kind == CalendarEventKind.exam && event.start.hour == 0
        ? 'exam'
        : '${event.start.hour.toString().padLeft(2, '0')}:${event.start.minute.toString().padLeft(2, '0')}';
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Material(
        color: const Color(0x29FFFFFF),
        borderRadius: BorderRadius.circular(AppRadii.chip),
        child: InkWell(
          borderRadius: BorderRadius.circular(AppRadii.chip),
          onTap: () => showEventDetailDialog(context, event, repository),
          child: Container(
            height: 20,
            padding: const EdgeInsets.only(left: 7, right: 7),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(AppRadii.chip),
              border: Border(left: BorderSide(color: accent, width: 3)),
            ),
            child: Row(
              children: [
                Text(
                  time,
                  style: const TextStyle(color: AppColors.muted, fontSize: 9.5, fontWeight: FontWeight.w600),
                ),
                const SizedBox(width: 6),
                Expanded(
                  child: Text(
                    event.title,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(color: Color(0xFFDFE7E7), fontSize: 10.5, fontWeight: FontWeight.w600),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _MoreLink extends StatelessWidget {
  const _MoreLink({required this.events, required this.repository});

  final List<CalendarEvent> events;
  final CalendarEventsRepository repository;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(left: 8, top: 1),
      child: InkWell(
        onTap: () => showDialog<void>(
          context: context,
          builder: (context) => SimpleDialog(
            backgroundColor: const Color(0xFF232E30),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(AppRadii.control + 2)),
            title: const Text('More events', style: TextStyle(color: AppColors.text, fontSize: 15)),
            children: [
              for (final ev in events)
                InkWell(
                  onTap: () {
                    Navigator.of(context).pop();
                    showEventDetailDialog(context, ev, repository);
                  },
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 8),
                    child: Row(
                      children: [
                        Container(width: 3, height: 18, color: accentFor(ev)),
                        const SizedBox(width: 10),
                        Expanded(child: Text(ev.title, style: const TextStyle(color: AppColors.text, fontSize: 12))),
                        Text(
                          '${ev.start.hour.toString().padLeft(2, '0')}:${ev.start.minute.toString().padLeft(2, '0')}',
                          style: const TextStyle(color: AppColors.muted, fontSize: 11),
                        ),
                      ],
                    ),
                  ),
                ),
            ],
          ),
        ),
        child: Text(
          '+${events.length} more',
          style: const TextStyle(color: AppColors.muted, fontSize: 10, fontWeight: FontWeight.w600),
        ),
      ),
    );
  }
}

class _SpanBanner extends StatelessWidget {
  const _SpanBanner({
    required this.event,
    required this.roundedLeft,
    required this.roundedRight,
    required this.onTap,
  });

  final CalendarEvent event;
  final bool roundedLeft;
  final bool roundedRight;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    const banner = Color(0xFF2C4A3F); // flat muted green (dark-minimal)
    return Material(
      color: banner,
      borderRadius: BorderRadius.horizontal(
        left: Radius.circular(roundedLeft ? AppRadii.control : 0),
        right: Radius.circular(roundedRight ? AppRadii.control : 0),
      ),
      child: InkWell(
        borderRadius: BorderRadius.horizontal(
          left: Radius.circular(roundedLeft ? AppRadii.control : 0),
          right: Radius.circular(roundedRight ? AppRadii.control : 0),
        ),
        onTap: onTap,
        child: Align(
          alignment: Alignment.centerLeft,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Text(
              event.title,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(color: Color(0xFF8FD4B4), fontSize: 10.5, fontWeight: FontWeight.w700),
            ),
          ),
        ),
      ),
    );
  }
}

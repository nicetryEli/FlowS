import 'package:flutter/material.dart';

import '../data/events_repository.dart';
import '../models/calendar_event.dart';
import '../theme.dart';
import '../widgets/event_detail_dialog.dart';
import '../widgets/event_style.dart';

/// Week view — flat dark time grid. Columns SUN→SAT, hourly gutter.
/// All-day/spanning events are NOT rendered here (owner: the all-day
/// banner row was removed); they live in the month view.
class WeekView extends StatefulWidget {
  const WeekView({super.key, required this.repository, required this.weekAnchor});

  final CalendarEventsRepository repository;
  final DateTime weekAnchor;

  @override
  State<WeekView> createState() => _WeekViewState();
}

class _WeekViewState extends State<WeekView> {
  static const int startHour = 6;
  static const int endHour = 24; // exclusive
  static const double hourHeight = 58;
  static const double gutterWidth = 48;

  late final ScrollController _scrollController;

  @override
  void initState() {
    super.initState();
    final now = DateTime.now();
    final offset = ((now.hour - startHour + now.minute / 60).clamp(0, endHour - startHour - 1)) * hourHeight;
    _scrollController = ScrollController(initialScrollOffset: offset);
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final weekStart = DateTime(
      widget.weekAnchor.year,
      widget.weekAnchor.month,
      widget.weekAnchor.day - (widget.weekAnchor.weekday % 7),
    );
    final days = List.generate(7, (i) => weekStart.add(Duration(days: i)));
    final now = DateTime.now();
    final isThisWeek = days.any((d) => d.year == now.year && d.month == now.month && d.day == now.day);

    final from = days.first;
    final to = days.last.add(const Duration(days: 1));
    final events = widget.repository.forRange(from, to);
    final timed = events.where((e) => !e.allDay).toList();

    return Column(
      children: [
        // ── Header: weekday + date per column ──────────────────────
        Padding(
          padding: const EdgeInsets.fromLTRB(26, 14, 26, 6),
          child: Row(
            children: [
              const SizedBox(width: gutterWidth),
              for (final d in days)
                Expanded(
                  child: _DayHeader(
                    date: d,
                    isToday: d.year == now.year && d.month == now.month && d.day == now.day,
                  ),
                ),
            ],
          ),
        ),

        // ── Scrollable time grid ────────────────────────────────────
        Expanded(
          child: LayoutBuilder(
            builder: (context, constraints) {
              final totalHeight = (endHour - startHour) * hourHeight;
              return Scrollbar(
                controller: _scrollController,
                child: SingleChildScrollView(
                  controller: _scrollController,
                  padding: const EdgeInsets.fromLTRB(26, 0, 26, 18),
                  child: SizedBox(
                    height: totalHeight,
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        // Hour gutter
                        SizedBox(
                          width: gutterWidth,
                          child: Stack(
                            children: [
                              for (int h = startHour; h < endHour; h++)
                                Positioned(
                                  top: (h == startHour ? 2 : (h - startHour) * hourHeight - 6),
                                  left: 0,
                                  right: 6,
                                  child: Text(
                                    '${h.toString().padLeft(2, '0')}:00',
                                    textAlign: TextAlign.right,
                                    style: const TextStyle(color: AppColors.muted2, fontSize: 9.5, fontWeight: FontWeight.w600),
                                  ),
                                ),
                            ],
                          ),
                        ),
                        // Day columns
                        for (var i = 0; i < 7; i++) ...[
                          if (i > 0) const SizedBox(width: 8),
                          Expanded(
                            child: _DayColumn(
                              date: days[i],
                              events: timed.where((e) => e.occursOn(days[i])).toList(),
                              repository: widget.repository,
                              startHour: startHour,
                              hourHeight: hourHeight,
                              isToday: isThisWeek && days[i].year == now.year && days[i].month == now.month && days[i].day == now.day,
                              showNowLine: isThisWeek && days[i].year == now.year && days[i].month == now.month && days[i].day == now.day,
                              nowMinutes: now.hour * 60 + now.minute,
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                ),
              );
            },
          ),
        ),
      ],
    );
  }
}

class _DayHeader extends StatelessWidget {
  const _DayHeader({required this.date, required this.isToday});

  final DateTime date;
  final bool isToday;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(
          weekdayShort(date),
          style: const TextStyle(color: AppColors.muted2, fontSize: 9.5, fontWeight: FontWeight.w700, letterSpacing: 1.6),
        ),
        const SizedBox(height: 4),
        Container(
          width: 34,
          height: 34,
          alignment: Alignment.center,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: isToday ? AppColors.white : Colors.transparent,
          ),
          child: Text(
            '${date.day}',
            style: TextStyle(
              color: isToday ? AppColors.slate : const Color(0xFFB9C4C5),
              fontSize: 15,
              fontWeight: FontWeight.w700,
            ),
          ),
        ),
      ],
    );
  }
}

class _DayColumn extends StatelessWidget {
  const _DayColumn({
    required this.date,
    required this.events,
    required this.repository,
    required this.startHour,
    required this.hourHeight,
    required this.isToday,
    required this.showNowLine,
    required this.nowMinutes,
  });

  final DateTime date;
  final List<CalendarEvent> events;
  final CalendarEventsRepository repository;
  final int startHour;
  final double hourHeight;
  final bool isToday;
  final bool showNowLine;
  final int nowMinutes;

  @override
  Widget build(BuildContext context) {
    final hours = <int>[for (int h = startHour; h < 24; h++) h];
    final lanes = _assignLanes(events);

    return DecoratedBox(
      decoration: BoxDecoration(
        color: AppColors.cell,
        borderRadius: BorderRadius.circular(AppRadii.cell),
      ),
      child: Stack(
        children: [
          // Hour grid lines
          for (var i = 1; i < hours.length; i++)
            Positioned(
              top: i * hourHeight,
              left: 0,
              right: 0,
              child: Container(height: 1, color: const Color(0x0AFFFFFF)),
            ),
          // Events
          LayoutBuilder(
            builder: (context, constraints) => Stack(
              children: [
                for (var i = 0; i < events.length; i++)
                  _PositionedEvent(
                    event: events[i],
                    lane: lanes[i],
                    totalLanes: _laneCount(events, lanes),
                    columnWidth: constraints.maxWidth,
                    startHour: startHour,
                    hourHeight: hourHeight,
                    repository: repository,
                  ),
              ],
            ),
          ),
          // Now line
          if (showNowLine && nowMinutes >= startHour * 60)
            Positioned(
              top: ((nowMinutes - startHour * 60) / 60) * hourHeight - 1,
              left: 0,
              right: 0,
              child: Row(
                children: [
                  Container(width: 7, height: 7, decoration: const BoxDecoration(color: AppColors.white, shape: BoxShape.circle)),
                  Expanded(child: Container(height: 1.5, color: AppColors.white.withValues(alpha: 0.55))),
                ],
              ),
            ),
        ],
      ),
    );
  }
}

/// Greedy lane assignment: each overlapping event gets its own column index.
List<int> _assignLanes(List<CalendarEvent> events) {
  final order = List.generate(events.length, (i) => i)
    ..sort((a, b) {
      final cmp = events[a].start.compareTo(events[b].start);
      return cmp != 0 ? cmp : events[a].end.compareTo(events[b].end);
    });
  final lanes = List.filled(events.length, 0);
  for (final i in order) {
    final e = events[i];
    var lane = 0;
    while (true) {
      final conflict = List.generate(events.length, (j) => j).any((j) =>
          j != i && lanes[j] == lane && events[j].start.isBefore(e.end) && e.start.isBefore(events[j].end));
      if (!conflict) break;
      lane++;
      if (lane > 8) break;
    }
    lanes[i] = lane;
  }
  return lanes;
}

int _laneCount(List<CalendarEvent> events, List<int> lanes) {
  if (events.isEmpty) return 1;
  return lanes.reduce((a, b) => a > b ? a : b) + 1;
}

class _PositionedEvent extends StatelessWidget {
  const _PositionedEvent({
    required this.event,
    required this.lane,
    required this.totalLanes,
    required this.columnWidth,
    required this.startHour,
    required this.hourHeight,
    required this.repository,
  });

  final CalendarEvent event;
  final int lane;
  final int totalLanes;
  final double columnWidth;
  final int startHour;
  final double hourHeight;
  final CalendarEventsRepository repository;

  @override
  Widget build(BuildContext context) {
    final startMinutes = event.start.hour * 60 + event.start.minute;
    var endMinutes = event.end.hour * 60 + event.end.minute;
    if (endMinutes <= startMinutes) endMinutes = startMinutes + 30; // deadline instant → 30-min block
    final top = ((startMinutes - startHour * 60) / 60) * hourHeight;
    var height = ((endMinutes - startMinutes) / 60) * hourHeight;
    if (height < 22) height = 22;

    final laneWidth = columnWidth / totalLanes;
    final accent = accentFor(event);

    return Positioned(
      top: top,
      left: lane * laneWidth + 2,
      width: laneWidth - 4,
      height: height,
      child: Material(
        color: accent.withValues(alpha: 0.16),
        borderRadius: BorderRadius.circular(AppRadii.chip),
        child: InkWell(
          borderRadius: BorderRadius.circular(AppRadii.chip),
          onTap: () => showEventDetailDialog(context, event, repository),
          child: Container(
            padding: const EdgeInsets.fromLTRB(6, 3, 5, 3),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(AppRadii.chip),
              border: Border(left: BorderSide(color: accent, width: 3)),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  event.kind == CalendarEventKind.deadline
                      ? '⏰ ${event.start.hour.toString().padLeft(2, '0')}:${event.start.minute.toString().padLeft(2, '0')}'
                      : '${event.start.hour.toString().padLeft(2, '0')}:${event.start.minute.toString().padLeft(2, '0')}',
                  style: TextStyle(color: accent, fontSize: 8.5, fontWeight: FontWeight.w800),
                ),
                Expanded(
                  child: Text(
                    event.title,
                    maxLines: 3,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(color: AppColors.text, fontSize: 10, fontWeight: FontWeight.w600, height: 1.15),
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

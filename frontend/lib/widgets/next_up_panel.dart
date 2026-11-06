import 'package:flutter/material.dart';

import '../data/mock_events.dart';
import '../models/calendar_event.dart';
import '../theme.dart';
import 'event_detail_dialog.dart';
import 'event_style.dart';

/// Right panel: "Next up this week" cards computed from the repository,
/// plus legend + the Add-personal-event CTA.
class NextUpPanel extends StatelessWidget {
  const NextUpPanel({super.key, required this.repository, required this.onAddPersonal});

  final MockEventsRepository repository;
  final VoidCallback onAddPersonal;

  @override
  Widget build(BuildContext context) {
    final now = DateTime.now();
    final upcoming = repository.events.where((e) => e.end.isAfter(now)).toList()
      ..sort((a, b) => a.start.compareTo(b.start));
    final next = upcoming.take(3).toList();
    final classes = repository.events.where((e) => e.kind == CalendarEventKind.classSession).length;
    final exams = repository.events.where((e) => e.kind == CalendarEventKind.exam).length;

    return Container(
      width: 292,
      color: AppColors.bgDeep,
      padding: const EdgeInsets.fromLTRB(18, 18, 18, 22),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _header('NEXT UP', 'THIS WEEK'),
          const SizedBox(height: 10),
          Row(
            children: [
              Container(
                width: 7,
                height: 7,
                decoration: const BoxDecoration(color: AppColors.green, shape: BoxShape.circle),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  'Synced · $classes sessions · $exams exams',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(color: AppColors.muted, fontSize: 10),
                ),
              ),
            ],
          ),
          const SizedBox(height: 14),
          if (next.isEmpty)
            const Text('Nothing scheduled — enjoy the break 🎓', style: TextStyle(color: AppColors.muted, fontSize: 12))
          else
            ...next.map((ev) => Padding(
                  padding: const EdgeInsets.only(bottom: 10),
                  child: _UpcomingCard(
                    event: ev,
                    onTap: () => showEventDetailDialog(context, ev, repository),
                  ),
                )),
          const SizedBox(height: 6),
          _header('LEGEND', null),
          const SizedBox(height: 10),
          const Wrap(spacing: 12, runSpacing: 8, children: [
            _LegendDot(color: AppColors.blue, label: 'Class'),
            _LegendDot(color: AppColors.red, label: 'Exam'),
            _LegendDot(color: AppColors.magenta, label: 'Deadline'),
            _LegendDot(color: AppColors.green, label: 'Personal'),
            _LegendDot(color: AppColors.orange, label: 'Makeup / học bù'),
            _LegendDot(color: AppColors.absentAccent, label: 'Absent / báo vắng'),
          ]),
          const Spacer(),
          SizedBox(
            height: 46,
            child: FilledButton.icon(
              onPressed: onAddPersonal,
              icon: const Icon(Icons.add_rounded, size: 18),
              label: const Text('Add personal event', style: TextStyle(fontWeight: FontWeight.w800)),
              style: FilledButton.styleFrom(
                backgroundColor: AppColors.slate,
                foregroundColor: AppColors.white,
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(AppRadii.control + 2)),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _header(String title, String? trailing) => Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            title,
            style: const TextStyle(
              color: AppColors.muted2,
              fontSize: 10.5,
              fontWeight: FontWeight.w800,
              letterSpacing: 1.8,
            ),
          ),
          if (trailing != null)
            Text(
              trailing,
              style: const TextStyle(color: AppColors.white, fontSize: 9, fontWeight: FontWeight.w800, letterSpacing: 1.3),
            ),
        ],
      );
}

class _UpcomingCard extends StatelessWidget {
  const _UpcomingCard({required this.event, required this.onTap});

  final CalendarEvent event;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final accent = accentFor(event);
    final rel = _relativeDay(event.start);
    final meta = <String>[];
    if (event.location != null) meta.add('📍 ${event.location}');
    if (event.startPeriod != null) meta.add('Tiết ${event.startPeriod}–${event.endPeriod}');
    if (event.examType != null) meta.add(event.examType!);

    return Material(
      color: AppColors.cell,
      borderRadius: BorderRadius.circular(AppRadii.control + 2),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(AppRadii.control + 2),
        child: Container(
          padding: const EdgeInsets.fromLTRB(12, 10, 12, 12),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(AppRadii.control + 2),
            border: Border(left: BorderSide(color: accent, width: 3)),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                '${_kindTag(event)} · $rel · ${event.start.hour.toString().padLeft(2, '0')}:${event.start.minute.toString().padLeft(2, '0')}',
                style: const TextStyle(
                  color: AppColors.muted,
                  fontSize: 8.5,
                  fontWeight: FontWeight.w800,
                  letterSpacing: 1.2,
                ),
              ),
              const SizedBox(height: 4),
              Text(
                event.title,
                style: const TextStyle(color: AppColors.text, fontSize: 12.5, fontWeight: FontWeight.w700, height: 1.32),
              ),
              if (meta.isNotEmpty) ...[
                const SizedBox(height: 5),
                Text(
                  meta.join('  ·  '),
                  style: const TextStyle(color: AppColors.muted, fontSize: 10.5),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  static String _kindTag(CalendarEvent ev) => switch (ev.kind) {
        CalendarEventKind.classSession => 'CLASS',
        CalendarEventKind.exam => 'EXAM',
        CalendarEventKind.deadline => 'DEADLINE',
        CalendarEventKind.personal => 'PERSONAL',
      };

  static String _relativeDay(DateTime d) {
    final now = DateTime.now();
    final today = DateTime(now.year, now.month, now.day);
    final day = DateTime(d.year, d.month, d.day);
    final delta = day.difference(today).inDays;
    return switch (delta) {
      0 => 'TODAY',
      1 => 'TOMORROW',
      _ => shortDate(d).toUpperCase(),
    };
  }
}

class _LegendDot extends StatelessWidget {
  const _LegendDot({required this.color, required this.label});

  final Color color;
  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(width: 9, height: 9, decoration: BoxDecoration(color: color, borderRadius: BorderRadius.circular(2))),
        const SizedBox(width: 6),
        Text(label, style: const TextStyle(color: AppColors.muted, fontSize: 10.5)),
      ],
    );
  }
}

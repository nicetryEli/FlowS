import 'package:flutter/material.dart';

import '../theme.dart';
import 'event_style.dart';

enum CalendarViewMode { week, month, year }

/// Top navigation bar: tabs on the left, view switch + month nav right.
/// Mirrors the approved prototype (YEAR intentionally removed).
class TopBar extends StatelessWidget {
  const TopBar({
    super.key,
    required this.mode,
    required this.onModeChanged,
    required this.anchorDate,
    required this.onPrev,
    required this.onNext,
    required this.onToday,
  });

  final CalendarViewMode mode;
  final ValueChanged<CalendarViewMode> onModeChanged;
  final DateTime anchorDate;
  final VoidCallback onPrev;
  final VoidCallback onNext;
  final VoidCallback onToday;

  static void _comingSoon(BuildContext context, String what) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('$what — coming soon'), duration: const Duration(seconds: 2)),
    );
  }

  String get _label => switch (mode) {
        CalendarViewMode.month => monthLabel(anchorDate),
        CalendarViewMode.year => '${anchorDate.year}',
        CalendarViewMode.week => () {
            final start = anchorDate.subtract(Duration(days: anchorDate.weekday % 7));
            final end = start.add(const Duration(days: 6));
            return '${_mon(start.month)} ${start.day} – ${_mon(end.month)} ${end.day}, ${end.year}';
          }(),
      };

  static String _mon(int m) => monthLabel(DateTime(2000, m)).split(' ').first.substring(0, 3);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(26, 16, 26, 0),
      child: SizedBox(
        height: 46,
        child: Row(
          children: [
            // Tabs (Scheduled is the only active screen)
            Row(
              children: [
                _tab(context, 'Activity'),
                _tab(context, 'Tasks'),
                _tab(context, 'Scheduled', active: true),
                _tab(context, 'Analytics'),
                _tab(context, 'Documents'),
              ],
            ),
            const Spacer(),
            // View switch
            Container(
              decoration: BoxDecoration(
                color: const Color(0xFF1A2325),
                borderRadius: BorderRadius.circular(999),
              ),
              padding: const EdgeInsets.all(4),
              child: Row(
                children: [
                  _viewButton('WEEK', CalendarViewMode.week),
                  const SizedBox(width: 2),
                  _viewButton('MONTH', CalendarViewMode.month),
                  const SizedBox(width: 2),
                  _viewButton('YEAR', CalendarViewMode.year),
                ],
              ),
            ),
            const SizedBox(width: 14),
            // Month navigation
            _navButton(Icons.chevron_left_rounded, onPrev),
            Container(
              height: 38,
              constraints: const BoxConstraints(minWidth: 150),
              margin: const EdgeInsets.symmetric(horizontal: 6),
              padding: const EdgeInsets.symmetric(horizontal: 12),
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: const Color(0xFF243033),
                borderRadius: BorderRadius.circular(AppRadii.control + 4),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    _label,
                    style: const TextStyle(
                      color: AppColors.text,
                      fontSize: 12,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  const SizedBox(width: 10),
                  const Icon(Icons.expand_more_rounded, size: 18, color: AppColors.muted),
                ],
              ),
            ),
            _navButton(Icons.chevron_right_rounded, onNext),
            TextButton(
              onPressed: onToday,
              style: TextButton.styleFrom(
                foregroundColor: AppColors.muted,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                minimumSize: const Size(0, 38),
              ),
              child: const Text('TODAY', style: TextStyle(fontSize: 10, fontWeight: FontWeight.w800, letterSpacing: 1.2)),
            ),
          ],
        ),
      ),
    );
  }

  Widget _tab(BuildContext context, String label, {bool active = false}) {
    return Padding(
      padding: const EdgeInsets.only(right: 8),
      child: InkWell(
        onTap: active ? null : () => _comingSoon(context, label),
        child: SizedBox(
          height: 46,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.start,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              // Active tab: white top indicator anchored to the tab bar top
              AnimatedContainer(
                duration: const Duration(milliseconds: 220),
                curve: Curves.easeOutCubic,
                height: 2,
                width: active ? 56 : 24,
                color: active ? AppColors.white : Colors.transparent,
              ),
              const SizedBox(height: 10),
              Text(
              label,
              style: TextStyle(
                color: active ? AppColors.white : AppColors.muted,
                fontSize: 13,
                fontWeight: FontWeight.w600,
              ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _viewButton(String label, CalendarViewMode value) {
    final active = mode == value;
    return GestureDetector(
      onTap: () => onModeChanged(value),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        curve: Curves.easeOutCubic,
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 7),
        decoration: BoxDecoration(
          color: active ? AppColors.slate : Colors.transparent,
          borderRadius: BorderRadius.circular(999),
        ),
        child: Text(
          label,
          style: TextStyle(
            color: active ? Colors.white : AppColors.muted,
            fontSize: 11,
            fontWeight: FontWeight.w700,
            letterSpacing: 1.0,
          ),
        ),
      ),
    );
  }

  Widget _navButton(IconData icon, VoidCallback onTap) {
    return Material(
      color: const Color(0xFF243033),
      borderRadius: BorderRadius.circular(AppRadii.control),
      child: InkWell(
        borderRadius: BorderRadius.circular(AppRadii.control),
        onTap: onTap,
        child: SizedBox(
          width: 34,
          height: 38,
          child: Icon(icon, color: AppColors.muted, size: 20),
        ),
      ),
    );
  }
}

import 'package:flutter/material.dart';

import '../theme.dart';

/// Left icon rail — mirrors the prototype. Calendar is the only active
/// destination for now; the rest show a "Coming soon" hint.
class AppRail extends StatelessWidget {
  const AppRail({super.key});

  static void _comingSoon(BuildContext context, String what) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('$what — coming soon'), duration: const Duration(seconds: 2)),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 64,
      color: AppColors.bgDeep,
      child: Column(
        children: [
          const SizedBox(height: 18),
          Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              color: AppColors.white,
              borderRadius: BorderRadius.circular(AppRadii.iconRail),
            ),
            alignment: Alignment.center,
            child: const Text(
              'TD',
              style: TextStyle(
                color: Color(0xFF082A33),
                fontSize: 15,
                fontWeight: FontWeight.w800,
                letterSpacing: 0.4,
              ),
            ),
          ),
          const SizedBox(height: 18),
          _RailButton(icon: Icons.home_rounded, onTap: () => _comingSoon(context, 'Home')),
          _RailButton(
            icon: Icons.calendar_month_rounded,
            active: true,
            onTap: () {}, // already on the calendar
          ),
          _RailButton(icon: Icons.menu_book_rounded, onTap: () => _comingSoon(context, 'Courses')),
          _RailButton(icon: Icons.timer_outlined, onTap: () => _comingSoon(context, 'Deadlines')),
          _RailButton(icon: Icons.edit_note_rounded, onTap: () => _comingSoon(context, 'Exams')),
          const Spacer(),
          _RailButton(icon: Icons.settings_outlined, onTap: () => _comingSoon(context, 'Settings')),
          const SizedBox(height: 6),
          Container(
            width: 36,
            height: 36,
            decoration: const BoxDecoration(color: AppColors.green, shape: BoxShape.circle),
            alignment: Alignment.center,
            child: const Text(
              'AL',
              style: TextStyle(color: Color(0xFF062A1C), fontWeight: FontWeight.w800, fontSize: 12),
            ),
          ),
          const SizedBox(height: 14),
        ],
      ),
    );
  }
}

class _RailButton extends StatelessWidget {
  const _RailButton({required this.icon, required this.onTap, this.active = false});

  final IconData icon;
  final VoidCallback onTap;
  final bool active;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Tooltip(
        message: active ? 'Calendar' : 'Coming soon',
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 180),
          curve: Curves.easeOutCubic,
          decoration: BoxDecoration(
            color: active ? AppColors.slate : Colors.transparent,
            borderRadius: BorderRadius.circular(AppRadii.iconRail),
          ),
          child: InkWell(
            borderRadius: BorderRadius.circular(AppRadii.iconRail),
            onTap: onTap,
            child: SizedBox(
              width: 40,
              height: 40,
              child: Icon(
                icon,
                size: 19,
                color: active ? AppColors.white : AppColors.muted,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

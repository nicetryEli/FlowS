import 'package:flutter/material.dart';

import '../theme.dart';

/// Footer strip — visual parity with the reference (links are inert).
class FooterStrip extends StatelessWidget {
  const FooterStrip({super.key});

  void _placeholder(BuildContext context, String what) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('$what — placeholder for the real page'), duration: const Duration(seconds: 2)),
    );
  }

  @override
  Widget build(BuildContext context) {
    const links = [
      'Privacy',
      'Community',
      'Help',
      'Cookie preferences',
      'Do not sell or share my personal information',
    ];
    return Container(
      height: 34,
      color: AppColors.bgDeep,
      padding: const EdgeInsets.symmetric(horizontal: 18),
      child: Row(
        children: [
          for (final link in links)
            Padding(
              padding: const EdgeInsets.only(right: 22),
              child: InkWell(
                onTap: () => _placeholder(context, link),
                child: Text(
                  link,
                  style: const TextStyle(color: AppColors.muted, fontSize: 10.5),
                ),
              ),
            ),
          const Spacer(),
          const Text('TDTU · Local only', style: TextStyle(color: AppColors.muted2, fontSize: 9.5, letterSpacing: 1.2)),
        ],
      ),
    );
  }
}

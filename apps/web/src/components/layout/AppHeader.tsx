'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useAuth } from '@/lib/auth';
import MobileNav from './MobileNav';

const navItems = [
  { href: '/dashboard', label: 'Dashboard' },
  { href: '/leads', label: 'Leads' },
  { href: '/emails', label: 'Emails' },
  { href: '/recordings', label: 'Recordings' },
  { href: '/profile', label: 'Profile' },
];

interface AppHeaderProps {
  variant?: 'default' | 'gradient';
}

export default function AppHeader({ variant = 'default' }: AppHeaderProps) {
  const pathname = usePathname();
  const { user, logout } = useAuth();

  const headerClass =
    variant === 'gradient'
      ? 'border-b border-white/5 bg-black/20 backdrop-blur-xl'
      : 'border-b border-border bg-card';

  const logoGradient =
    variant === 'gradient'
      ? 'bg-gradient-to-br from-indigo-500 to-purple-600 shadow-lg shadow-indigo-500/25'
      : 'bg-primary';

  const textColor = variant === 'gradient' ? 'text-white' : 'text-foreground';
  const mutedColor = variant === 'gradient' ? 'text-white/60' : 'text-muted-foreground';
  const hoverBg = variant === 'gradient' ? 'hover:bg-white/5' : 'hover:bg-muted';
  const activeBg = variant === 'gradient' ? 'bg-white/10' : 'bg-muted';

  return (
    <header className={`${headerClass} sticky top-0 z-40`}>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-14 sm:h-16 items-center">
          {/* Logo */}
          <Link href="/dashboard" className="flex items-center space-x-2 sm:space-x-3">
            <div
              className={`w-8 h-8 sm:w-9 sm:h-9 rounded-lg sm:rounded-xl ${logoGradient} flex items-center justify-center`}
            >
              <span className="text-white font-bold text-base sm:text-lg">O</span>
            </div>
            <span
              className={`text-lg sm:text-xl font-bold ${
                variant === 'gradient'
                  ? 'bg-gradient-to-r from-white to-white/60 bg-clip-text text-transparent'
                  : ''
              }`}
            >
              Outreach
            </span>
          </Link>

          {/* Desktop Navigation */}
          <nav className="hidden md:flex items-center space-x-1">
            {navItems.map((item) => {
              const isActive = pathname === item.href || pathname?.startsWith(item.href + '/');

              return (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`px-3 lg:px-4 py-2 rounded-lg text-sm font-medium transition-all ${
                    isActive
                      ? `${activeBg} ${textColor}`
                      : `${mutedColor} ${hoverBg} hover:${textColor}`
                  }`}
                >
                  {item.label}
                </Link>
              );
            })}
          </nav>

          {/* Desktop User Menu */}
          <div className="hidden md:flex items-center space-x-4">
            <span className={`text-sm ${mutedColor} truncate max-w-[150px]`}>
              {user?.email}
            </span>
            <button
              onClick={() => logout()}
              className={`text-sm ${mutedColor} hover:${textColor} transition-colors`}
            >
              Logout
            </button>
          </div>

          {/* Mobile Navigation */}
          <MobileNav />
        </div>
      </div>
    </header>
  );
}

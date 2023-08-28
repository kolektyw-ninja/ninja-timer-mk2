import React, { useState } from 'react';
import classNames from 'classnames';
import { Navbar } from './Navbar';
import { Timers } from './Timers';
import Notifications from './Notifications';
import { Settings } from './Settings';

export const App = () => {
  const [isDark, setIsDark] = useState(window.matchMedia('(prefers-color-scheme: dark)'));

  return (
    <div className={classNames({ dark: isDark }, "h-full", "w-screen")}>
      <div className='h-full w-screen dark:bg-slate-900'>
        <div className="container mx-auto">
          {/* <Navbar darkMode={isDark} setDarkMode={setIsDark} /> */}
          <Notifications />
          <Timers />
          <Settings />
        </div>
      </div>
    </div>
  );
};

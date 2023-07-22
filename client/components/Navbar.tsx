import React from 'react';
import { FaGear } from 'react-icons/fa6';
import { Toggle } from './Toggle';

export const Navbar = ({ darkMode, setDarkMode }) => {
  return (
    <nav className='w-full p-4 bg-slate-700 mb-4 rounded-b-md font-medium text-gray-400'>
      <ul className=''>
        <li className='flex flex-row justify-between items-center'>
          <FaGear />
          <Toggle label="dark mode" checked={darkMode} onChange={e => setDarkMode(e.target.checked)} />
        </li>
      </ul>
    </nav>
  );
};

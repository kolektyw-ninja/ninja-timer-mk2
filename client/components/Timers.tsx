import React, { useCallback, useEffect, useRef, useState, JSX } from 'react';
import { FaPlay, FaStop, FaArrowRotateLeft } from 'react-icons/fa6';
import Api, { Timer } from '../api'

const timeElapsed = (timer: Timer): number | null => timer.startedAt == null ? null : Date.now() - timer.startedAt - timer.countdown * 1000
const useTimerRef = (timer: Timer | null) => {
  const timerRef = useRef()
  const requestRef = useRef()

  const animate = useCallback(() => {
    if (timer == null || timer.startedAt == null) {
      return
    }

    timerRef.current.innerHTML = formatTime(timeElapsed(timer!)!)
    requestRef.current = requestAnimationFrame(animate)
  }, [timer])

  useEffect(() => {
    if (timer == null || timer.startedAt == null) {
      timerRef.current.innerHTML = formatTime(0)
    } else if (timer.state == "Stopped") {
      timerRef.current.innerHTML = timer.formatted
    } else { 
      requestRef.current = requestAnimationFrame(animate)
      return () => cancelAnimationFrame(requestRef.current)
    }
  }, [timer])

  return timerRef
}

const useApiTimers = (): (Timer | null)[] => {
  const [timers, setTimers] = useState([null] as (Timer | null)[])

  const updateTimers = useCallback((e: CustomEvent<Timer[]>) => {
    const timers = e.detail
    setTimers(timers)
  }, [setTimers])

  useEffect(() => {
    window.addEventListener("timerUpdate", updateTimers)
    return () => window.removeEventListener("timerUpdate", updateTimers)
  }, [updateTimers])

  return timers
}

export const formatTime = (milis: number): string => {
  if (milis < 0) {
    const countdown = Math.ceil(Math.abs(milis) / 1000)
    return `${countdown}`
  }
  const minutes = Math.floor(milis / 1000 / 60)
  const seconds = Math.floor(milis % 60000 / 1000)
  const millis = Math.floor(milis % 1000)

  return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}.${millis.toString().padStart(3, "0")}`
}

type ButtonProps = {
  onClick?: () => void,
  enabled?: boolean,
  children: JSX.Element[],
}

const Button: React.FC<ButtonProps> = ({ onClick, disabled, children }) => {
  return (
    <button onClick={onClick} disabled={disabled} className="px-4 py-2 text-sm inline-flex flex-grow justify-center items-center font-medium disabled:cursor-not-allowed text-gray-900 bg-white border border-gray-200 first-child:rounded-l-md last-child:rounded-r-md hover:bg-gray-100 hover:text-blue-700 focus:z-10 focus:rbng-2 focus:ring-blue-700 focus:text-blue-700 dark:bg-gray-700 disabled:dark:bg-gray-800 dark:border-gray-600 dark:text-white dark:hover:text-white dark:hover:enabled:bg-gray-600 dark:focus:ring-blue-500 dark:focus:text-white">
      {children}
    </button>
  )
}

export const Timers = () => {
  const apiTimers = useApiTimers()
  const timerRefs = apiTimers.map(timer => useTimerRef(timer))
  const startEnabled = apiTimers[0] && apiTimers[0].state == "Reset"
  const stopEnabled = apiTimers[0] && apiTimers[0].state != "Reset" && apiTimers[0].state != "Stopped"
  const resetEnabled = apiTimers[0] && apiTimers[0].state != "Reset"

  return (
    <div className="bg-gray-800 flex flex-col mx-auto items-center max-w-lg rounded-md shadow border border-slate-600">
      <div className="p-5">
        {timerRefs.map((ref, i) => <p key={i} className="dark:text-gray-300 font-mono p-3 text-5xl [&:not(:first-child)]:border-t [&:not(:first-child)]:border-t-slate-600" ref={ref} />)}
        {/* <p className="dark:text-gray-300 font-mono border-t border-t-slate-600 p-3 text-5xl" ref={timerRef2}></p> */}
      </div>
      <div className="inline-flex shadow-sm w-full" role="group">
        <Button onClick={Api.start} disabled={!startEnabled}>
          <FaPlay />
          <span className='ml-2'>Start</span>
        </Button>
        <Button onClick={Api.stop} disabled={!stopEnabled}>
          <FaStop />
          <span className='ml-2'>Stop</span>
        </Button>
        <Button onClick={Api.reset} disabled={!resetEnabled}>
          <FaArrowRotateLeft />
          <span className='ml-2'>Reset</span>
        </Button>
      </div>
    </div>
  );
};

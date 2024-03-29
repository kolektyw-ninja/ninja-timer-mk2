import React, { useState, useCallback, useRef } from 'react';
import * as Api from '../api';

export const Settings = () => {
    const backgroundRef = useRef();

    const changeBackground = useCallback((e) => {
        if (backgroundRef.current == null) {
            return
        }

        const data = new FormData()
        data.append("background", e.target.files[0])
        fetch("/api/upload_background", {
          method: "POST",
          body: data,
        })

    }, [backgroundRef])

    const [countdown, setCountdown] = useState(0)

    const saveCallback = useCallback(() => {
        fetch("/api/set_countdown", {
            method: "POST",
            headers: {
              "Content-Type": "application/json"
            },
            body: JSON.stringify({ countdown })
          })
    }, [countdown])

    return (
        <div className="block p-6 mt-5 max-w-lg mx-auto bg-white border border-gray-200 rounded-lg shadow hover:bg-gray-100 dark:bg-gray-800 dark:border-gray-700 dark:hover:bg-gray-700">
            <label className="block mb-2 text-sm font-medium text-gray-900 dark:text-white" for="user_avatar">Upload background</label>
            <input ref={backgroundRef} onChange={changeBackground} className="block w-full text-sm text-gray-900 border border-gray-300 rounded-lg cursor-pointer bg-gray-50 dark:text-gray-400 focus:outline-none dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400" aria-describedby="user_avatar_help" id="user_avatar" type="file"></input>
            <label className="block mb-2 text-sm font-medium text-gray-900 dark:text-white" for="user_avatar">Countdown</label>
            <input onChange={(e) => setCountdown(parseInt(e.target.value))} value={countdown} type="number" min="0" id="first_name" className="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" />
            <button onClick={saveCallback} type="button" className="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800">Save</button>
        </div>
    )
}

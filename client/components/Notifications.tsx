import React, { useState, useEffect, useCallback } from "react"
import Api from '../api'

type NotificationProps = {
  message: string,
}

const Notification: React.FC<NotificationProps> = (props: NotificationProps) => {
  return (
    <div className="p-4 mb-4 text-sm text-red-800 rounded-lg bg-red-50 dark:bg-gray-800 dark:text-red-400" role="alert">
      {props.message}
    </div>
  )
}

const useConnected = () => {
  const [connected, setConnected] = useState(Api.isConnected)

  const callback = useCallback(() => {
    setConnected(Api.isConnected)
  }, [setConnected])

  useEffect(() => {
    window.addEventListener("connectionChanged", callback)
    return callback
  }, [setConnected])

  return connected
}

const Notifications: React.FC = () => {
  const connected = useConnected()

  const notifications = []
  return (
    <div>
      {connected ? null : <Notification message="Connection lost. Reconnecting..."/>}
    </div>
  )
}

export default Notifications

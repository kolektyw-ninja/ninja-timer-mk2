const post = async (url: string) => {
    return fetch(url, { method: "POST" })
}

const start = () => {
    post("/api/start_timer")
}

const stop = () => {
    post("/api/stop_timer")
}

const reset = () => {
    post("/api/reset_timer")
}

const dispatchEvent = (name: string, data: object) => {
    window.dispatchEvent(new CustomEvent(name, {detail: data}))
}

export type Timer = {
    id: number,
    startedAt: number | null,
    stoppedAt: number | null,
    countdown: number,
    state: "Reset" | "CountingDown" | "Running" | "Stopped",
    formatted: string,
}

type TimersEvent = {
    now: number,
    timers: Timer[],
}

const connect = () => {

    const startEventSource = () => {
        const source = new EventSource("/api/events")
        let keepAliveInternal: number | null = null
        let lastPing = Date.now()

        source.addEventListener("open", () => {
            Api.isConnected = true
            dispatchEvent("connectionChanged", {})
            keepAliveInternal = setInterval(() => {
                const timeSincePing = Date.now() - lastPing
                if (timeSincePing > 11000) {
                    source.close()
                    // noConnectionMessage.classList.remove("is-hidden")
                    // setupEvents()
                    Api.isConnected = false
                    dispatchEvent("connectionChanged", {})
                    clearInterval(keepAliveInternal!)
                    source.close()
                    setTimeout(startEventSource, 1000)
                }
            }, 1000)

            post("/api/request_sync")
        })

        source.addEventListener("error", () => {
            console.log("connection lost")
            source.close()
            setTimeout(startEventSource, 1000)
        })

        source.addEventListener("syncTimers", e => {
            const data = JSON.parse(e.data) as TimersEvent

            data.timers.forEach(timer => dispatchEvent("timerUpdate", timer))
        })

        source.addEventListener("syncSettings", e => {
            const data = JSON.parse(e.data)
            console.log(data)
        })

        source.addEventListener("ping", e => {
            lastPing = Date.now()
        })
    }

    startEventSource()

// const setupEvents = () => {
//   events = new EventSource("/api/events")

//   events.addEventListener("open", _ => {
//     noConnectionMessage.classList.add("is-hidden")
//     clearInterval(keepAliveInterval)

//     lastPing = Date.now()
//     keepAliveInternal = setInterval(_ => {
//       const timeSincePing = Date.now() - lastPing
//       if (timeSincePing > 11000) {
//         events.close()
//         noConnectionMessage.classList.remove("is-hidden")
//         setupEvents()
//       }
//     }, 1000)
//   })

//   events.addEventListener("error", _ => {
//     noConnectionMessage.classList.remove("is-hidden")
//   })

//   events.addEventListener("syncTimers", e => {
//     const data = JSON.parse(e.data)
//     const newTimer = data.timer

//     if (newTimer.startedAt != null) {
//       newTimer.startedAt = Date.parse(newTimer.startedAt)
//     }

//     if (newTimer.stoppedAt != null) {
//       newTimer.stoppedAt = Date.parse(newTimer.stoppedAt)
//     }

//     timer = newTimer
//     timerClockOffset = Date.parse(data.now) - Date.now()

//     updateButtons()
//   })


//   events.addEventListener("ping", _ => {
//     lastPing = Date.now()
//   })

//   events.addEventListener("syncSettings", e => {
//     const data = JSON.parse(e.data)
//     settings = data.settings

//     updateSettings()
//   })

//   fetch("/api/request_sync", {
//     method: "POST",
//   })
// }
}

const Api =  { 
    connect,
    start,
    stop,
    reset,
    isConnected: false,
}

export default Api

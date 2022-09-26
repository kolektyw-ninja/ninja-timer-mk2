const start = document.getElementById("start")
const stop = document.getElementById("stop")
const reset = document.getElementById("reset")

const timerDisplay = document.getElementById("timerDisplay")

const events = new EventSource("/api/events")

let timer = null
let timerClockOffset = 0

const updateButtons = () => {
  if (timer == null || timer.state == "Reset") {
    start.disabled = false
    stop.disabled = true
    reset.disabled = true
  } else if (timer.state == "Stopped") {
    start.disabled = true
    stop.disabled = true
    reset.disabled = false
  } else {
    start.disabled = true
    stop.disabled = false
    reset.disabled = false
  }
}

const formatTimer = () => {
  const now = Date.now() + timerClockOffset
  const timeElapsed = now - timer.startedAt
  const adjustedTime = timeElapsed - timer.countdown * 1000

  const isCountdown = adjustedTime < 0

  if (isCountdown) {
    return Math.ceil(-adjustedTime/1000).toString()
  } else {
    const hundredths = Math.floor((adjustedTime / 10) % 100)
    const seconds = Math.floor((adjustedTime / 1000) % 60)
    const minutes = Math.floor(adjustedTime / (1000 * 60))

    return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}.${hundredths.toString().padStart(2, "0")}`
  }
}

const updateTimerDisplay = () => {
  if (timer == null || timer.state == "Reset") {
    timerDisplay.innerHTML = "00:00.00"
  } else if (timer.state == "Stopped") {
    timerDisplay.innerHTML = timer.formatted
  } else {
    timerDisplay.innerHTML = formatTimer()
  }
}

events.addEventListener("syncTimers", e => {
  const data = JSON.parse(e.data)
  const newTimer = data.timer

  if (newTimer.startedAt != null) {
    newTimer.startedAt = Date.parse(newTimer.startedAt)
  }

  if (newTimer.stoppedAt != null) {
    newTimer.stoppedAt = Date.parse(newTimer.stoppedAt)
  }

  timer = newTimer
  timerClockOffset = Date.parse(data.now) - Date.now()

  updateButtons()
})

events.addEventListener("syncSettings", e => {
  const data = JSON.parse(e.data)
  console.log(data)
})

fetch("/api/request_sync", {
  method: "POST",
})

start.addEventListener("click", e => {
  e.preventDefault()
  fetch("/api/start_timer", { method: "POST" })
})

stop.addEventListener("click", e => {
  e.preventDefault()
  fetch("/api/stop_timer", { method: "POST" })
})

reset.addEventListener("click", e => {
  e.preventDefault()
  fetch("/api/reset_timer", { method: "POST" })
})

const settingsForm = document.getElementById("settings")

settings.addEventListener("submit", e => {
  e.preventDefault()
})

const frameCallback = () => {
  updateTimerDisplay()
  requestAnimationFrame(frameCallback)
}

frameCallback()

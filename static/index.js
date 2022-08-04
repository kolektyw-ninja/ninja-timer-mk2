const start = document.getElementById("start")
const stop = document.getElementById("stop")
const reset = document.getElementById("reset")

const events = new EventSource("/api/events")

events.addEventListener("stateChanged", e => {
  console.log(e.data)
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

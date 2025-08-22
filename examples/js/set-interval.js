console.log("Sync start")

let i = 0
let interval = setInterval(() => {
    console.log(`Interval ${i}`)

    if (i === 10) {
        clearInterval(interval)
    }

    i += 1
}, 1000)

console.log("Sync end")

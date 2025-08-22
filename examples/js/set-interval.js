console.log("Sync start")

let i = 0
let interval = setInterval(() => {
    console.log(`Interval ${i}`)

    if (i === 5) {
        clearInterval(interval)
    }

    i += 1
}, 500)

console.log("Sync end")

let function factorial {
    labels[return(Integer), generic<>, mutable(false), visibility(public),
        scope(project), implementation(full), ]

    parameters {
        let n value {
            labels[return(Integer), mutable(false), scope(local), implementation(full), ]
        }
    }
    body {
        return if n = 0 then 1 else n * factorial(n - 1);
    }
}

let function main {
    labels[return(), generic<>, mutable(false), visibility(public),
        scope(project), implementation(full), ]
    parameters{}
    body { printLine(factorial(15)); }
}

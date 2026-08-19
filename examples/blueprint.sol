Cat -> {
    parameters(
        name,
            is value, type String, instance scope,
    )
    speak -> { printLine("Meow") }
        is function, returns nothing, instance scope,
}
    is blueprint,

main -> {
    cat -> Cat("Whiskers")
        is value, type Cat, local scope,
    cat.speak()
    printLine(cat.name)
}
    is function, returns nothing,

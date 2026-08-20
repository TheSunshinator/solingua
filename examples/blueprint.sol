Cat -> {
    parameters(
        name,
            is value, type String, instance scope,
    )
    speak -> {
        parameters()
        printLine("Meow")
    }
        is function, returns nothing, instance scope,
}
    is blueprint,

main -> {
    parameters()
    cat -> Cat("Whiskers")
        is value, type Cat, local scope,
    cat.speak()
    printLine(cat.name)
}
    is function, returns nothing, project scope,

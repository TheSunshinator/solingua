Animal -> {
    name
        is value, type String, instance scope
    speak -> {
        parameters()
    }
        is function, returns nothing, instance scope
}
    is blueprint, declared implementation,

Cat -> {
    parameters(
        name,
            is value, type String, instance scope, full implementation,
            contracted,
    )
    speak -> {
        parameters()
        printLine("Meow")
    }
        is function, returns nothing, instance scope, full implementation,
        contracted,
}
    is blueprint, full implementation, type Animal,

Dog -> {
    parameters(
        name,
            is value, type String, instance scope, full implementation,
            contracted,
    )
    speak -> {
        parameters()
        printLine("Woof")
    }
        is function, returns nothing, instance scope, full implementation,
        contracted,
}
    is blueprint, full implementation, type Animal,

main -> {
    parameters()
    cat -> Cat("Whiskers")
        is value, type Cat, local scope,
    dog -> Dog("Simba")
        is value, type Dog, local scope,
    cat.speak()
    dog.speak()
    printLine(cat.name)
    printLine(dog.name)
}
    is function, returns nothing, project scope,

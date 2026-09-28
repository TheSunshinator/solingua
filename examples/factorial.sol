let 𝑓 factorial {
    #return(Integer) #generic⟨⟩ #mutable(false) #visibility(public)
    #scope(project) #implementation(full);

    parameters
    let value n {
        #return(Integer) #mutable(false) #scope(local) #implementation(full);
    }

    body
    return if n = 0 then 1 else n × factorial(n - 1);
}

let 𝑓 main {
    #return() #generic⟨⟩ #mutable(false) #visibility(public)
    #scope(project) #implementation(full);

    parameters

    body
    printLine(factorial(15));
}

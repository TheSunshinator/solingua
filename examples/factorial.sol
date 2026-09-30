let 𝑓 factorial {
    #return(Integer) #generic⟨⟩ #visibility(public)
    #scope(project) #implementation(full);

    parameters
    let data n {
        #return(Integer) #mutable(false) #scope(local) #implementation(full);
    }

    body
    return if n = 0 then 1 else n × factorial(n - 1);
}

let 𝑓 main {
    #return() #generic⟨⟩ #visibility(public)
    #scope(project) #implementation(full);

    parameters

    body
    printLine(factorial(15));
}

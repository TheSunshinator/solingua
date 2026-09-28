let type Cat {
    #return() #generic⟨⟩ #mutable(false) #visibility(public)
    #scope(project) #implementation(full);

    parameters
    let value name {
        #return(String) #mutable(false) #scope(instance) #implementation(full);
    }

    instance
    let 𝑓 speak {
        #return() #generic⟨⟩ #mutable(false) #visibility(public)
        #scope(instance) #implementation(full) #contract();

        parameters

        body
        printLine("Meow");
    }
}

let type Point {
    #return() #generic⟨⟩ #mutable(false) #visibility(public)
    #scope(project) #implementation(full);

    parameters
    let value x {
        #return(Integer) #mutable(true) #scope(instance) #implementation(full);
    }
    let value y {
        #return(Integer) #mutable(true) #scope(instance) #implementation(full);
    }

    instance
    let 𝑓 moveBy {
        #return() #generic⟨⟩ #mutable(false) #visibility(public)
        #scope(instance) #implementation(full) #contract();

        parameters
        let value dx {
            #return(Integer) #mutable(false) #scope(local) #implementation(full);
        }
        let value dy {
            #return(Integer) #mutable(false) #scope(local) #implementation(full);
        }

        body
        x becomes x + dx;
        y becomes y + dy;
    }
}

let 𝑓 main {
    #return() #generic⟨⟩ #mutable(false) #visibility(public)
    #scope(project) #implementation(full);

    parameters

    body
    let value cat {
        #return(Cat) #mutable(false) #scope(local) #implementation(full);
        initially Cat("Krokmou")
    }
    cat → speak();
    printLine(cat → name);

    let value p {
        #return(Point) #mutable(false) #scope(local) #implementation(full);
        initially Point(10, 20)
    }
    p → moveBy(5, -3);
    printLine(p → x);
    printLine(p → y);
}

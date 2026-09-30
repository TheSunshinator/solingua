let type Cat {
    #return() #generic⟨⟩ #extensible(closed) #visibility(public)
    #scope(project) #implementation(full);

    parameters
    let data name {
        #return(String) #mutable(false) #scope(instance) #implementation(full);
    }

    instance
    let 𝑓 speak {
        #return() #generic⟨⟩ #extensible(closed) #visibility(public)
        #scope(instance) #implementation(full) #contract();

        parameters

        body
        printLine("Meow");
    }
}

let type Point {
    #return() #generic⟨⟩ #extensible(closed) #visibility(public)
    #scope(project) #implementation(full);

    parameters
    let data x {
        #return(Integer) #mutable(true) #scope(instance) #implementation(full);
    }
    let data y {
        #return(Integer) #mutable(true) #scope(instance) #implementation(full);
    }

    instance
    let 𝑓 moveBy {
        #return() #generic⟨⟩ #extensible(closed) #visibility(public)
        #scope(instance) #implementation(full) #contract();

        parameters
        let data dx {
            #return(Integer) #mutable(false) #scope(local) #implementation(full);
        }
        let data dy {
            #return(Integer) #mutable(false) #scope(local) #implementation(full);
        }

        body
        x becomes x + dx;
        y becomes y + dy;
    }
}

let 𝑓 main {
    #return() #generic⟨⟩ #visibility(public)
    #scope(project) #implementation(full);

    parameters

    body
    let data cat {
        #return(Cat) #mutable(false) #scope(local) #implementation(full);
        initially Cat("Krokmou")
    }
    cat → speak();
    printLine(cat → name);

    let data p {
        #return(Point) #mutable(false) #scope(local) #implementation(full);
        initially Point(10, 20)
    }
    p → moveBy(5, -3);
    printLine(p → x);
    printLine(p → y);
}

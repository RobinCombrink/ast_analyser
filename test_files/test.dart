void main(){ 
    dynamic dyn = 43;

    int number = dyn as int;
    
    if (number is double) {
        print ("true");
    }

    print(number);
    
}
void main(){ 
    dynamic dyn = 44;
    String number = dyn as String;
    
    if (number is double) {
        print ("true");
    }

    print(number);
    
}
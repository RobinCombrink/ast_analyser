void main() {
  String? string = maybeString("abc");
  String number = string! + "a";
  print(number);

  String? string1 = maybeString("abc");
  String number2 = string1! + "a";
  print(number2);
}

String? maybeString(String? string) {
  return string;
}


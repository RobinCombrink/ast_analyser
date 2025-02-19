void main() {

  String? string = maybeString("abc");
  String number = string! + "a";
  print(number);

  String? differentString = maybeString("abc");
  String number2 = differentString! + "a";
  print(number2);
}

String? maybeString(String? string) {
  return string;
}


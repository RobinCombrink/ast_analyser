void main() {
  String? string = maybeString("abc");
  String number = string! + "a";
  print(number);
}

String? maybeString(String? string) {
  return string;
}


use std::io;

pub fn run_calculator() {
    let mut memory: Option<f64> = None;

    println!("\nЛаскаво просимо до калькулятора!\nОкрім виразів, можна написати 'exit' для виходу \
        або скористатися словом 'mem' замість числа, щоб взяти результат попереднього обчислення\n");
    loop {
        println!("Введіть вираз:");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Не вдалося зчитати введення");

        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        // пробуємо обчислити
        match evaluate_expression(&input, &memory) {
            Ok(result) => {
                println!("Результат: {}", result);
                memory = Some(result);  // зберігаємо в пам'ять
            }
            Err(e) => {
                println!("Помилка: {}", e);
            }
        }
    }
}

fn evaluate_expression(input: &str, memory: &Option<f64>) -> Result<f64, &'static str> {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    let mut output: Vec<f64> = Vec::new();
    let mut operators: Vec<&str> = Vec::new();

    // if tokens.len() != 3 {
    //     return Err("Невірний формат. Використовуйте: число оператор число (наприклад, 3 + 2 або mem + 2). \
    //     Пробіли мають значення");
    // }

    // обробляємо токени
    for token in tokens {
        match token {
            "mem" => {
                match memory {
                    Some(value) => output.push(*value),
                    None => return Err("Пам'ять порожня"),
                }
            }
            op if is_operator(op) => {
                // порівнюємо пріоритет оператора, на який наткнулися, з пріоритетом оператора на вершині стеку
                // якщо нижче: то треба виконати оператор на вершині стеку операторів для двох останніх чисел на вершині стеку чисел
                while let Some(&top_op) = operators.last() {
                    if priority(op) <= priority(top_op) {
                        let b = output.pop().ok_or("Недостатньо значень у виразі")?;
                        let a = output.pop().ok_or("Недостатньо значень у виразі")?;
                        let result = apply_operator(top_op, a, b)?;
                        output.push(result);
                        operators.pop();
                    } else {
                        break;
                    }
                }
                operators.push(op);
            }
            number => {
                let parsed_number: f64 = number.parse::<f64>().expect("Невірний формат числа");
                output.push(parsed_number);
            }
        }
    }

    while let Some(op) = operators.pop() {
        let b = output.pop().ok_or("Недостатньо значень у виразі")?;
        let a = output.pop().ok_or("Недостатньо значень у виразі")?;
        let result = apply_operator(op, a, b)?;
        output.push(result);
    }

    output.pop().ok_or("Невірний формат виразу")


}

fn is_operator(token: &str) -> bool {
    matches!(token, "+" | "-" | "*" | "/")
}


fn priority(operator: &str) -> i32 {
    match operator {
        "+" | "-" => 1,
        "*" | "/" => 2,
        _ => 0,
    }
}


fn apply_operator(operator: &str, num1: f64, num2: f64) -> Result<f64, &'static str> {
    match operator {
        "+" => Ok(num1 + num2),
        "-" => Ok(num1 - num2),
        "*" => Ok(num1 * num2),
        "/" => {
            if num2 == 0.0 {
                Err("Ділення на нуль не допускається")
            } else {
                Ok(num1 / num2)
            }
        }
        _ => Err("Невідомий оператор"),
    }
}

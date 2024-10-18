use std::io;

pub fn run_calculator() {
    let mut memory: Option<f64> = None;

    println!("\nЛаскаво просимо до калькулятора!\nОкрім виразів, можна написати 'exit' для виходу \
        або скористатися словом 'mem' замість числа, щоб взяти результат попереднього обчислення\n");

    println!("Виберіть режим введення: ");
    println!("1 - Звичайний");
    println!("2 - Польський реверсний (ПОЛІЗ / RPN)");

    let mut mode = String::new();

    loop {
        mode.clear();

        io::stdin().read_line(&mut mode).expect("Не вдалося зчитати введення");

        let mode = mode.trim();

        if !matches!(mode, "1" | "2") {
            println!("Немає такої опції! Напишіть 1 або 2")
        }
        else {
            break;
        }
    }

    loop {
        println!("Введіть вираз:");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Не вдалося зчитати введення");

        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        let result = if mode.contains("1") {
            evaluate_expression(&input, &memory)
        } else {
            evaluate_rpn_expression(&input, &memory)
        };

        // пробуємо обчислити
        match result {
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

fn evaluate_rpn_expression(input: &str, memory: &Option<f64>) -> Result<f64, &'static str> {
    let mut stack: Vec<f64> = Vec::new();

    for token in input.split_whitespace() {
        match token {
            "+" => {
                let b = stack.pop().ok_or("Недостатньо значень у стеці")?;
                let a = stack.pop().ok_or("Недостатньо значень у стеці")?;
                stack.push(a + b);
            },
            "-" => {
                let b = stack.pop().ok_or("Недостатньо значень у стеці")?;
                let a = stack.pop().ok_or("Недостатньо значень у стеці")?;
                stack.push(a - b);
            },
            "*" => {
                let b = stack.pop().ok_or("Недостатньо значень у стеці")?;
                let a = stack.pop().ok_or("Недостатньо значень у стеці")?;
                stack.push(a * b);
            },
            "/" => {
                let b = stack.pop().ok_or("Недостатньо значень у стеці")?;
                let a = stack.pop().ok_or("Недостатньо значень у стеці")?;
                if b == 0.0 {
                    return Err("Ділення на нуль не допускається");
                }
                stack.push(a / b);
            },
            "mem" => {
                let number = match memory {
                    Some(value) => *value,
                    None => return Err("Пам'ять порожня"),
                };
                stack.push(number);
            },
            _ => {
                let number = token.parse::<f64>().expect("Невірний формат числа");
                stack.push(number);
            },
        }
    }

    stack.pop().ok_or("Вираз не обчислено")
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

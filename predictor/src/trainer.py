from datetime import datetime

import torch
from torch.cuda.amp import autocast
from torch import nn

from sample import MAX_WINDOW_SIZE, BYTES_PER_ENTRY


class EarlyExit:
    def __init__(self, tolerence):
        self.tolerence = tolerence
        self.bad_rounds = 0
        self.last_losses = torch.as_tensor([])

    def mean_loss(self):
        if self.last_losses.size(dim=0) == 0:
            return None

        return self.last_losses.mean()

    def append_loss(self, loss):
        self.last_losses = torch.cat((self.last_losses, torch.as_tensor([loss])))
        self.last_losses = self.last_losses[-8:]

    def update(self, loss):
        mean_loss = self.mean_loss()
        print("Mean validation loss: ", mean_loss)

        if mean_loss is None:
            self.append_loss(loss)
            return

        if loss > mean_loss:
            self.bad_rounds += 1
        else:
            self.bad_rounds = 0
            self.append_loss(loss)

        if self.bad_rounds == self.tolerence:
            raise Exception("early exit because validation loss stopped going down")


def train(data_loader, validation_loader, load_fn, model_dir, load_path, device):

    early_exit = EarlyExit(32)

    cpu = torch.device("cpu")

    print("Train called with: ", model_dir, load_path)

    # Send none to load_fn if load_path is None otherwise append the model dir to it
    path = None

    if load_path != None:
        path = model_dir + load_path

    command_generator, optimizer, scheduler_base, scheduler_step = load_fn(path, device)
    criterion = nn.CrossEntropyLoss()
    running_loss = torch.zeros(1, device=device)

    def step(ldr, backprop):

        print("Starting batch")
        running_loss.zero_()

        count = 0

        for seq in iter(ldr):

            seq = seq.to(device)
            inputs = seq[:, :-BYTES_PER_ENTRY]
            labels = seq[:, BYTES_PER_ENTRY:]

            optimizer.zero_grad()
            logits = command_generator(inputs)

            with torch.cuda.amp.autocast():
                loss = criterion(logits, labels)

            if backprop:
                loss.backward()
                optimizer.step()
            running_loss.add_(loss.detach())

            count += 1

        result = running_loss / count
        return result

    def save(name):
        print("Saving model to path: " + name)
        torch.save(command_generator.state_dict(), "./" + name + ".model")
        torch.save(optimizer.state_dict(), "./" + name + ".optimizer")

    epoch = 1

    while True:

        print("Pre-step LR:", optimizer.param_groups[0]["lr"])

        # Do a ROUND_SZ of training and backprop
        loss = step(data_loader, True)

        # Do a round 10 of validation with no backprop
        validation_loss = step(validation_loader, False)

        # Update scheduler based on validation loss
        scheduler_step.step()
        scheduler_base.step(validation_loss)

        early_exit.update(validation_loss.item())

        print("Loss:", loss.item())
        print("Validation loss:", validation_loss.item())
        print("LR:", optimizer.param_groups[0]["lr"])

        print("Saving checkpoint")

        # Timestamp every 10th epoch to test fits later
        if epoch % 3 == 0:
            save(model_dir + "/" + str(int(datetime.now().timestamp())))

        save(model_dir + "/last.checkpoint")

        print("Saved checkpoint")

        epoch += 1

    return command_generator.eval()

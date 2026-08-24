require "a"
# ^ function.method.builtin

class Car < Vehicle
  # <- keyword
  #    ^ constructor

  def init(id)
    # <- keyword
    # ^ function.method

    @id = id
    # <- property
    #     ^ variable.parameter

    yield
    # <- keyword
    return
    # <- keyword
    next
    # <- keyword
  end

  private
  # ^ function.method.builtin

  public
  # ^ function.method.builtin

  protected
  # ^ function.method.builtin
end
# <- keyword
